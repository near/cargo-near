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

<details>
  <summary>Install prebuilt binaries via shell script (Linux, macOS)</summary>

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/cargo-near/releases/latest/download/cargo-near-installer.sh | sh
```
</details>

<details>
  <summary>Install prebuilt binaries via powershell script (Windows)</summary>

```sh
irm https://github.com/near/cargo-near/releases/latest/download/cargo-near-installer.ps1 | iex
```
</details>

<details>
  <summary>Install prebuilt binaries into your Node.js application</summary>

```sh
npm install cargo-near
```
</details>

<details>
  <summary>Compile and install from source code (Cargo)</summary>

```sh
cargo install --locked cargo-near
```

or, install the most recent version from git repository:

```sh
$ git clone https://github.com/near/cargo-near
$ cargo install --locked --path cargo-near
```
</details>

## Usage

See `cargo near --help` for a complete list of available commands or run `cargo near` to dive into interactive mode. Help is also available for each individual command with a `--help` flag, e.g. `cargo near build --help`.

```console
cargo near
```

Starts interactive mode that will allow to explore all the available commands.

---
Additionally depends on [Git](https://git-scm.com/) binary being installed, besides [cargo](https://github.com/rust-lang/cargo).

```console
cargo near new
```

Initializes a new project skeleton to create a contract from a template.

[Example](./docs/workflows.md) of github [workflows](./cargo-near/src/commands/new/new-project-template/.github/workflows) configuration, created by `cargo near new`.

---

```console
cargo near build
```

Builds a NEAR smart contract along with its [ABI](https://github.com/near/abi) (while in the directory containing contract's Cargo.toml).

By default, this runs a reproducible build in a [Docker](https://docs.docker.com/) container, which:

1. runs against source code version, committed to git, ignoring any uncommitted changes
2. requires that `Cargo.lock` of project is created (e.g. via `cargo update`) and added to git. 
    - this enables `--locked` build by downstream `cargo` command. 
3. will use configuration in [`[package.metadata.near.reproducible_build]`](https://github.com/near/cargo-near/blob/main/cargo-near/src/commands/new/new-project-template/Cargo.toml.template#L14-L25) 
   section of contract's `Cargo.toml` and [`package.repository`](https://github.com/near/cargo-near/blob/main/cargo-near/src/commands/new/new-project-template/Cargo.toml.template#L9) field
    - default values for this section can also be found in `Cargo.toml` of 
      template project, generated by `cargo near new`

Important flags:

1. `--no-docker`
    - flag can be used to perform a regular build with rust toolchain installed onto host, running the `cargo-near` cli. 
    - *NO*-Docker builds run against actual state of code in filesystem and not against a version, committed to source control.   

2. `--no-locked` 
    - flag is allowed in *NO*-Docker builds, e.g. to generate a `Cargo.lock` *and* simultaneously build the contract.
    - flag is allowed in Docker builds, but 
      - such builds are not reproducible due to potential update of dependencies and compiled `wasm` mismatch as the result.

---

```console
cargo near abi
```

Generates NEAR smart contract's [ABI](https://github.com/near/abi) (while in the directory containing contract's Cargo.toml).

Once contract is deployed, this will allow you to call a view function `__contract_abi` to retrieve a [ZST](https://facebook.github.io/zstd/)-compressed ABI.

---

```console
cargo near create-dev-account
```

Guides you through creation of a new NEAR account on [testnet](https://explorer.testnet.near.org).

---

```console
cargo near deploy
```

Builds the smart contract (equivalent to `cargo near build`) and guides you to deploy it to the blockchain.

By default, this runs a reproducible build in a Docker container. 

`deploy` command from Docker build requires that contract's source code:

1. doesn't have any modified tracked files, any staged changes or any untracked content.  
2. has been pushed to remote repository, identified by 
   [`package.repository`](https://github.com/near/cargo-near/blob/main/cargo-near/src/commands/new/new-project-template/Cargo.toml.template#L9).

Important flags:

1. `--no-docker`
    - flag can be used to perform a regular *NO*-Docker build *and* deploy. 
      - Similar to `build` command,  in this case none of the git-related concerns and restrictions apply.

2. `--no-locked` 
    - flag is declined for deploy, due to its effects on `build` result

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
