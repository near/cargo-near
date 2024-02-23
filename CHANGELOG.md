# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1](https://github.com/near/cargo-near/compare/cargo-near-v0.6.0...cargo-near-v0.6.1) - 2024-02-23

### Other
- Updated near-sdk-rs to version 5.0.0 for the new projects ([#132](https://github.com/near/cargo-near/pull/132))

## [0.6.0](https://github.com/near/cargo-near/compare/cargo-near-v0.5.2...cargo-near-v0.6.0) - 2024-02-03

### Added
- Use hello-world contract instead of the status-message contract for the new project starter
- Enable by default release mode, embedded ABIs with doc strings

## [0.5.2](https://github.com/near/cargo-near/compare/cargo-near-v0.5.1...cargo-near-v0.5.2) - 2024-01-27

### Other
- Updated "feature flag" for near-cli-rs (ledger) ([#126](https://github.com/near/cargo-near/pull/126))
- Updated near-sdk-rs to 5.0.0-alpha.2 in the new project template ([#127](https://github.com/near/cargo-near/pull/127))

## [0.5.1](https://github.com/near/cargo-near/compare/cargo-near-v0.5.0...cargo-near-v0.5.1) - 2024-01-25

### Other
- Upgraded NEAR crates to 0.20.0 release ([#125](https://github.com/near/cargo-near/pull/125))
- Updated binary releases pipeline to use cargo-dist v0.7.2 (previously v0.3.0)  ([#122](https://github.com/near/cargo-near/pull/122))

## [0.5.0](https://github.com/near/cargo-near/compare/cargo-near-v0.4.1...cargo-near-v0.5.0) - 2023-11-20

### Added
- New command to initialize a new smart contract project ([#117](https://github.com/near/cargo-near/pull/117))

### Other
- update `near-sdk`, `near-abi`, `borsh` version ([#109](https://github.com/near/cargo-near/pull/109))

## [0.4.1](https://github.com/near/cargo-near/compare/cargo-near-v0.4.0...cargo-near-v0.4.1) - 2023-10-19

### Added
- New command - deploy ([#113](https://github.com/near/cargo-near/pull/113))
- New command - create-dev-account ([#108](https://github.com/near/cargo-near/pull/108))

### Fixed
- `cargo near build` now works on Windows ([#110](https://github.com/near/cargo-near/pull/110))

### Other
- remove `#[ignore]` from parts of test suite, using `near-workspaces` ([#111](https://github.com/near/cargo-near/pull/111))

## [0.4.0](https://github.com/near/cargo-near/compare/cargo-near-v0.3.1...cargo-near-v0.4.0) - 2023-10-01

### Other
- [**breaking**] Re-implemented cargo-near to use interactive-clap and near-cli-rs features ([#103](https://github.com/near/cargo-near/pull/103))

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
