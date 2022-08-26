# cargo-near

Cargo extension for building [near-sdk-rs](https://github.com/near/near-sdk-rs) smart contracts and [ABI schemas](https://github.com/near/abi) on NEAR

To install:
```
cargo install --path cargo-near
```

To generate an [ABI](https://github.com/near/abi) for a contract (while in the directory containing contract's Cargo.toml):
```
cargo near abi
```

Or explicitly specify path to the Cargo manifest:
```
cargo near abi --manifest-path path/to/Cargo.toml
```