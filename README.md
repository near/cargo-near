<!-- markdownlint-disable MD041 -->

# cargo-near

Cargo extension for building [near-sdk-rs](https://github.com/near/near-sdk-rs) smart contracts and [ABI schemas](https://github.com/near/abi) on NEAR

## Usage

To build a NEAR smart contract (while in the directory containing contract's Cargo.toml):

```console
$ cargo near build
```

To generate an [ABI](https://github.com/near/abi) for a contract (while in the directory containing contract's Cargo.toml):

```console
$ cargo near abi
```

See `cargo near --help` for a complete list of options.

## Installation

From crates.io:

```console
$ cargo install cargo-near
```

To install from source:

```console
$ git clone https://github.com/near/cargo-near
$ cargo install --path cargo-near
```

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as below, without any additional terms or conditions.

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
