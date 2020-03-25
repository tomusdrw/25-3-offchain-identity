### 1. Clone repo:
```bash
$ git clone --depth 1 --branch ci-release-2.0.0-alpha.5+6 https://github.com/paritytech/substrate
```

### 2. Add offchain worker that prints `Hello World!'
Compile & run
```bash
$ cargo run -p node-template -- --dev -lruntime=trace
```

### 3. Print some logs from off-chain worker.
```rust
fn offchain_worker(number: T::BlockNumber) {
  use frame_support::debug;

  debug::warn!("Hello World from offchain workers!");
  debug::warn!("Current Block Number: {:?}", number);
}
```
