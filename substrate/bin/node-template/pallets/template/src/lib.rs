#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use codec::{Encode, Decode};
use frame_support::weights::SimpleDispatchInfo;
use frame_support::{decl_module, decl_storage, decl_error, dispatch, debug};
use lite_json::json::JsonValue;
use sp_runtime::offchain::http;
use sp_runtime::transaction_validity::{ValidTransaction, InvalidTransaction, TransactionValidity};
use sp_std::prelude::*;
use system::offchain::SubmitUnsignedTransaction;
use system::{ensure_signed, ensure_none};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;

	/// The type to submit unsigned transactions.
	type SubmitUnsignedTransaction:
		SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
}

type GistId = [u8; 32];
type GithubUsername = Vec<u8>;
type GistFilename = Vec<u8>;

#[derive(Encode, Decode)]
pub struct Request<Account> {
	pub account: Account,
	pub gist_id: GistId,
}

// This pallet's storage items.
decl_storage! {

	trait Store for Module<T: Trait> as TemplateModule {
		/// A map of requested Gist ids by particular account.
		Requests get(fn requests):
			map hasher(twox_64_concat) T::AccountId => Option<Request<T::AccountId>>;

		/// A map of already validated usernames.
		Usernames get(fn usernames):
			map hasher(twox_64_concat) T::AccountId => Option<GithubUsername>;
	}

}

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
		pub fn request_verification(origin, gist_id: GistId) -> dispatch::DispatchResult {
			// Retrieve sender of the transaction.
			let who = ensure_signed(origin)?;
			// TODO [ToDr] Disallow if already requested?
			// Perhaps best would be to let the request timeout at some point.
			// So that an account can request again in the future.
			// Also perhaps disallow if mapping exists?
			Requests::<T>::insert(who.clone(), Request {
				account: who,
				gist_id,
			});

			Ok(())
		}

		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
		pub fn respond_verification(origin, account_id: T::AccountId, username: GithubUsername)
			-> dispatch::DispatchResult
		{
			// We are sending unsigned transaction.
			ensure_none(origin)?;

			if !Requests::<T>::contains_key(&account_id) {
				Err("No request for this account.")?
			}

			// make sure to remove the request.
			// TODO [ToDr] Most likely we should check that we are responding to the same
			// request (i.e. include gist_id and check it here)
			Requests::<T>::remove(&account_id);

			// Insert to usernames.
			Usernames::<T>::insert(account_id, username);

			// TODO [ToDr] Dispatch event.
			Ok(())
		}

		fn offchain_worker(number: T::BlockNumber) {
			debug::warn!("Hello World from offchain workers!");
			debug::warn!("Current Block Number: {:?}", number);

			let res: Result<usize, &'static str> = (||{
				let mut count = 0;
				for Request { account, gist_id } in Requests::<T>::iter_values() {
					let (filename, username) = Self::retrieve_gist(&gist_id)?;
					debug::info!(
						"[{:?}] Retrieved:\nFilename: {:?}\nUsername: {:?}",
						gist_id, filename, username
					);
					Self::check_if_valid(&account, &filename)?;
					Self::send_response(account, username)?;
					count += 1;
				}
				Ok(count)
			})();

			// make an http request (request fixed login)
			match res {
				Ok(count) => debug::info!("Processed {} requests.", count),
				Err(err) => debug::error!("Unable to process: {}", err),
			}
		}
	}
}

impl<T: Trait> Module<T> {
	fn retrieve_gist(gist_id: &GistId)
		-> Result<(GistFilename, GithubUsername), &'static str>
	{
		let mut address = b"https://api.github.com/gists/".to_vec();
		address.extend(gist_id.as_ref());

		let address = sp_std::str::from_utf8(&address).unwrap();

		debug::info!("Requesting {}", address);
		let request = http::Request::get(address);
		let pending = request.send().map_err(|_| "Unable to send HTTP request")?;
		let response = pending.wait().map_err(|_| "HTTP failed")?;
		if response.code != 200 {
			Err("Unexpected response code. Perhaps the Gist does not exist.")?;
		}

		let body = response.body().collect::<Vec<u8>>();
		let body_str = sp_std::str::from_utf8(&body).map_err(|_| "Body not UTF8")?;
		let val = lite_json::parse_json(body_str).map_err(|_| "Unable to parse JSON")?;

		let files = get_object(&val, "files")?;
		let filename = &files.get(0).as_ref().ok_or_else(|| "malformed JSON")?.0;
		let filename = filename.iter().map(|c| *c as u8).collect();
		let username = get_string(get_object(&val, "owner")?, "login")?;

		Ok((filename, username))
	}

	fn check_if_valid(account_id: &T::AccountId, filename: &GistFilename)
		-> Result<(), &'static str>
	{
		let acc = account_id.encode();
		if &acc == filename {
			Ok(())
		} else {
			debug::warn!("Expected: {:?}, got: {:?}", acc, filename);
			Err("Invalid filename.")
		}
	}

	fn send_response(account_id: T::AccountId, username: GithubUsername)
		-> Result<(), &'static str>
	{

		let call = Call::respond_verification(account_id, username);
		T::SubmitUnsignedTransaction::submit_unsigned(call)
			.map_err(|_| "Unable to send transaction")
	}
}

impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;
	fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
		// Firstly let's check that we call the right function.
		if let Call::respond_verification(account_id, _username) = call {
			if !Requests::<T>::contains_key(&account_id) {
				return InvalidTransaction::Stale.into()
			}

			Ok(ValidTransaction {
				priority: (1 << 20),
				requires: vec![],
				provides: vec![codec::Encode::encode(&("github::identity", account_id))],
				longevity: 5,
				propagate: true,
			})
		} else {
			InvalidTransaction::Call.into()
		}
	}
}

type JsonObject = Vec<(Vec<char>, JsonValue)>;
fn get_object<'a>(val: &'a JsonValue, key: &str) -> Result<&'a JsonObject, &'static str> {
	if let JsonValue::Object(ref obj) = *val {
		if let JsonValue::Object(ref v) = *find_key(obj, key)? {
			return Ok(v)
		}
	}
	Err("Non-object on path.")
}

fn get_string(val: &JsonObject, key: &str) -> Result<Vec<u8>, &'static str> {
	if let JsonValue::String(ref chars) = find_key(val, key)? {
		Ok(chars.iter().map(|c| *c as u8).collect())
	} else {
		Err("String key not found in the object.")
	}
}

fn find_key<'a>(val: &'a JsonObject, key: &str) -> Result<&'a JsonValue, &'static str> {
	let chars = key.chars().collect::<Vec<_>>();
	for (ok, ov) in val {
		if ok == &chars {
			return Ok(ov)
		}
	}

	Err("Key not found in the object")
}
