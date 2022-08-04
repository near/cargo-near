# cargo-near
Cargo extension for building Rust smart contracts on NEAR

To install:
```
cargo install --path cargo-near
```

To generate ABI for a contract (while in the directory containing contract's Cargo.toml):
```
cargo near abi
```

Or explicitly specify path to the Cargo manifest:
```
cargo near abi --manifest-path path/to/Cargo.toml
```