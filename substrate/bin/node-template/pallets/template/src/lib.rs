#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{decl_module, decl_storage, decl_error, dispatch, debug};
use frame_support::weights::SimpleDispatchInfo;
use system::ensure_signed;
use codec::{Encode, Decode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
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
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
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
			-> dispatch::DispatchResult {
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

			// make an http request (request fixed login)
			match Self::process_requests() {
				Ok(count) => debug::info!("Processed {} requests.", count),
				Err(err) => debug::error!("Unable to process: {}", err),
			}
		}
	}
}

impl<T: Trait> Module<T> {
	fn process_requests() -> Result<usize, &'static str> {
		let mut count = 0;
		for Request { account, gist_id } in Requests::<T>::iter_values() {
			let (filename, username) = Self::retrieve_gist_filename(&gist_id)?;
			debug::info!(
				"[{:?}] Retrieved:\nFilename: {:?}\nUsername: {:?}",
				gist_id, filename, username
			);
			Self::check_if_valid(&account, &filename)?;
			Self::send_response(account, username)?;
			count += 1;
		}
		Ok(count)
	}

	fn retrieve_gist_filename(gist_id: &GistId)
		-> Result<(GistFilename, GithubUsername), &'static str>
	{
		unimplemented!()
	}

	fn check_if_valid(account_id: &T::AccountId, filename: &GistFilename)
		-> Result<(), &'static str>
	{
		unimplemented!()
	}

	fn send_response(account_id: T::AccountId, username: GithubUsername)
		-> Result<(), &'static str>
	{
		unimplemented!()
	}
}
