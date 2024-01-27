# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
