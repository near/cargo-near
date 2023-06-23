# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1] - 2023-06-23

- Exposed `build` and `abi` modules to make them reusable when cargo-near is used as a crate. <https://github.com/near/cargo-near/pull/97>

## [0.3.0] - 2022-11-10

Highlight: We revised the overall experience of the CLI, making it more accessible, robust, and easier to understand.

- The minimum supported version of the SDK for this release is `4.1.0`.
- Upgraded the `near-abi` version to `0.3.0`. <https://github.com/near/cargo-near/pull/83>
- The exported and embedded ABI now includes build information. <https://github.com/near/cargo-near/pull/55>
- When building a contract, the exported ABI now also includes the code hash of the built contract. <https://github.com/near/cargo-near/pull/55>
- Fixed a situation where `cargo-near` could potentially run into segfaults when working with incompatible versions of the SDK. <https://github.com/near/cargo-near/pull/74>
- `cargo-near` now only accepts valid UTF-8 input from the CLI, and will error out if it encounters invalid UTF-8. <https://github.com/near/cargo-near/pull/76>
- `cargo-near` no longer requires explicitly activating the `abi` feature for the SDK. <https://github.com/near/cargo-near/pull/85>
- Fixed a bug where `cargo-near` exports an empty ABI file when the target directory is explicitly specified. <https://github.com/near/cargo-near/pull/75>
- Introduced build stages with a neat report interface. <https://github.com/near/cargo-near/pull/59>, <https://github.com/near/cargo-near/pull/63>, <https://github.com/near/cargo-near/pull/69>
- Added the `--color` flag to control the color output. <https://github.com/near/cargo-near/pull/86>
- Ensured all forwarded `cargo` output retains colors in it's report, maintaining tooling familiarity. <https://github.com/near/cargo-near/pull/66>
- Removed the buffering that made `cargo`'s `stdout` lag behind its `stderr`. <https://github.com/near/cargo-near/pull/65>
- When building contracts, `cargo`'s warnings are only emitted at the build stage, and not duplicated. <https://github.com/near/cargo-near/pull/68>

## [0.2.0] - 2022-09-01

> Release Page: <https://github.com/near/cargo-near/releases/tag/v0.2.0>

[unreleased]: https://github.com/near/cargo-near/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/near/cargo-near/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/near/cargo-near/releases/tag/v0.2.0
