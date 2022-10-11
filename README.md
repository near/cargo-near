<!-- markdownlint-disable MD014 -->

<div align="center">

  <h1><code>cargo-near</code></h1>

  <p>
    <strong>Cargo extension for building <a href="https://github.com/near/near-sdk-rs">near-sdk-rs</a> smart contracts and <a href="https://github.com/near/abi">ABI schemas</a> on NEAR</strong>
  </p>

  <p>
    <a href="https://github.com/near/cargo-near/actions/workflows/test.yml?query=branch%3Amain"><img src="https://github.com/near/cargo-near/actions/workflows/test.yml/badge.svg" alt="Github CI Build" /></a>
    <a href="https://crates.io/crates/cargo-near"><img src="https://img.shields.io/crates/v/cargo-near.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/cargo-near"><img src="https://img.shields.io/crates/d/cargo-near.svg?style=flat-square" alt="Download" /></a>
  </p>

</div>

## Release notes

**Release notes and unreleased changes can be found in the [CHANGELOG](CHANGELOG.md)**

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
