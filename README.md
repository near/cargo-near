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

### Install prebuilt binaries via shell script (Linux, macOS)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/cargo-near/releases/latest/download/cargo-near-installer.sh | sh
```

### Install prebuilt binaries via powershell script (Windows)

```sh
irm https://github.com/near/cargo-near/releases/latest/download/cargo-near-installer.ps1 | iex
```

### Install prebuilt binaries into your npm project

```sh
npm install cargo-near
```

### Compile latest released version from source code

```console
cargo install cargo-near
```

### Compile latest development version from source code

```console
$ git clone https://github.com/near/cargo-near
$ cargo install --path cargo-near
```

## Usage

See `cargo near --help` for a complete list of available commands or run `cargo near` to dive into interactive mode. Help is also available for each individual command with a `--help` flag, e.g. `cargo near build --help`.

```console
cargo near
```

Starts interactive mode that will allow to explore all the available commands.

```console
cargo near build
```

Builds a NEAR smart contract along with its [ABI](https://github.com/near/abi) (while in the directory containing contract's Cargo.toml).

You can also make this command embed ABI into your WASM artifact by adding `--embed-abi` parameter. Once deployed, this will allow you to call a view function `__contract_abi` to retrieve a [ZST](https://facebook.github.io/zstd/)-compressed ABI.

```console
cargo near abi
```

Generates NEAR smart contract's [ABI](https://github.com/near/abi) (while in the directory containing contract's Cargo.toml).

```console
cargo near create-dev-account
```

Guides you through creation of a new NEAR account on [testnet](https://explorer.testnet.near.org).

```console
cargo near deploy
```

Builds the smart contract (equivalent to `cargo near build`) and guides you to deploy it to the blockchain.


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
