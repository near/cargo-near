# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.16.1](https://github.com/near/cargo-near/compare/cargo-near-v0.16.0...cargo-near-v0.16.1) - 2025-07-08

### Other

- Use re-exported `style` (indicatif) module from `tracing_indicatif` ([#352](https://github.com/near/cargo-near/pull/352))
- Fixed linting errors - non-inlined formatting syntax ([#354](https://github.com/near/cargo-near/pull/354))
- command to deploy non-reproducible in template README file ([#349](https://github.com/near/cargo-near/pull/349))
- update `cargo near new` template `image` and `image_digest` ([#347](https://github.com/near/cargo-near/pull/347))

## [0.16.0](https://github.com/near/cargo-near/compare/cargo-near-v0.15.0...cargo-near-v0.16.0) - 2025-05-19

### Added

- `--variant <name>` flag ([#339](https://github.com/near/cargo-near/pull/339))
- add `Feature::TruncSat` and `Feature::BulkMemory` to `wasm_opt::OptimizationOptions` ([#338](https://github.com/near/cargo-near/pull/338))

### Other

- Updated `near-cli` install in template workflows to 0.20.0; updated `sign-with-plaintext-private-key` cli flag -> argument usage ([#346](https://github.com/near/cargo-near/pull/346))
- update `cargo near new` template `image` and `image_digest` ([#345](https://github.com/near/cargo-near/pull/345))

## [0.15.0](https://github.com/near/cargo-near/compare/cargo-near-v0.14.2...cargo-near-v0.15.0) - 2025-05-16

### Other

- updates near-* dependencies to 0.30 release ([#341](https://github.com/near/cargo-near/pull/341))
- use 1.86.0 toolchain for contracts tests with live node ([#340](https://github.com/near/cargo-near/pull/340))
- `cargo-near-build` crate badge + install cli in integration tests (for `test_cargo_test_on_generated_project`) ([#337](https://github.com/near/cargo-near/pull/337))
- update `cargo near new` template `image` and `image_digest` ([#335](https://github.com/near/cargo-near/pull/335))

## [0.14.2](https://github.com/near/cargo-near/compare/cargo-near-v0.14.1...cargo-near-v0.14.2) - 2025-05-03

### Added

- [cargo_near_build::build_with_cli] method in `build_external` default feature ([#333](https://github.com/near/cargo-near/pull/333))
- [cargo_near_build::extended::build_with_cli] for build.rs of factories ([#334](https://github.com/near/cargo-near/pull/334))

### Other

- update `cargo near new` template `image` and `image_digest` ([#331](https://github.com/near/cargo-near/pull/331))

## [0.14.1](https://github.com/near/cargo-near/compare/cargo-near-v0.14.0...cargo-near-v0.14.1) - 2025-04-21

### Fixed

- doc publish. Change reference to `latest` instead of specific version ([#329](https://github.com/near/cargo-near/pull/329))

## [0.14.0](https://github.com/near/cargo-near/compare/cargo-near-v0.13.6...cargo-near-v0.14.0) - 2025-04-21

### Added

- [**breaking**] pass in external nep330_wasm_output path from env as override ([#328](https://github.com/near/cargo-near/pull/328))
- populate `output_wasm_path` into `ContractSourceMetadata` ([#323](https://github.com/near/cargo-near/pull/323))

### Other

- update `cargo near new` template `image` and `image_digest` ([#327](https://github.com/near/cargo-near/pull/327))

## [0.13.6](https://github.com/near/cargo-near/compare/cargo-near-v0.13.5...cargo-near-v0.13.6) - 2025-04-08

### Added

- extract `near-verify-rs` dependency ([#320](https://github.com/near/cargo-near/pull/320))

### Other

- update `cargo near new` template `image` and `image_digest` ([#318](https://github.com/near/cargo-near/pull/318))

## [0.13.5](https://github.com/near/cargo-near/compare/cargo-near-v0.13.4...cargo-near-v0.13.5) - 2025-03-20

### Other

- updates near-* dependencies to 0.29 release ([#314](https://github.com/near/cargo-near/pull/314))
- update `cargo near new` template `image` and `image_digest` : 0.13.4-rust-1.85.1 ([#315](https://github.com/near/cargo-near/pull/315))
- update `cargo near new` template `image` and `image_digest` ([#310](https://github.com/near/cargo-near/pull/310))
- update `cargo near new` template `image` and `image_digest` ([#306](https://github.com/near/cargo-near/pull/306))
- fix clippy 1.85 ([#311](https://github.com/near/cargo-near/pull/311))

## [0.13.4](https://github.com/near/cargo-near/compare/cargo-near-v0.13.3...cargo-near-v0.13.4) - 2025-02-13

### Added

- embed docs for flags/arguments for `-h`/`--help` (#304)

### Other

- update `cargo near new` template `image` and `image_digest` ([#300](https://github.com/near/cargo-near/pull/300))
- update `cargo near new` template `image` and `image_digest` ([#298](https://github.com/near/cargo-near/pull/298))

## [0.13.3](https://github.com/near/cargo-near/compare/cargo-near-v0.13.2...cargo-near-v0.13.3) - 2025-01-22

### Other

- update near-cli-rs to 0.18.0 (#293)
- update `cargo near new` template `image` and `image_digest` ([#288](https://github.com/near/cargo-near/pull/288))
- update `cargo near new` template `image` and `image_digest` ([#283](https://github.com/near/cargo-near/pull/283))

## [0.13.2](https://github.com/near/cargo-near/compare/cargo-near-v0.13.1...cargo-near-v0.13.2) - 2024-12-19

### Other

- near-* crates to 0.28 (#279)
- update `cargo near new` template `image` and `image_digest` ([#273](https://github.com/near/cargo-near/pull/273))

## [0.13.1](https://github.com/near/cargo-near/compare/cargo-near-v0.13.0...cargo-near-v0.13.1) - 2024-12-18

### Other

- update `cargo near new` template `image` and `image_digest` ([#269](https://github.com/near/cargo-near/pull/269))

## [0.13.0](https://github.com/near/cargo-near/compare/cargo-near-v0.12.2...cargo-near-v0.13.0) - 2024-12-17

### Added

- reproducible choice interactive (#262)

### Other

- update `cargo near new` template `image` and `image_digest` ([#259](https://github.com/near/cargo-near/pull/259))
- update `cargo near new` template `image` and `image_digest` ([#257](https://github.com/near/cargo-near/pull/257))

## [0.12.2](https://github.com/near/cargo-near/compare/cargo-near-v0.12.1...cargo-near-v0.12.2) - 2024-11-20

### Other

- update `cargo near new` template `image` and `image_digest` ([#253](https://github.com/near/cargo-near/pull/253))

## [0.12.1](https://github.com/near/cargo-near/compare/cargo-near-v0.12.0...cargo-near-v0.12.1) - 2024-11-19

### Other

- updates near-* dependencies to 0.27 release ([#251](https://github.com/near/cargo-near/pull/251))
- update `cargo near new` template `image` and `image_digest` ([#250](https://github.com/near/cargo-near/pull/250))

## [0.12.0](https://github.com/near/cargo-near/compare/cargo-near-v0.11.0...cargo-near-v0.12.0) - 2024-11-14

### Added

- Added the ability to use the TEACH-ME mode ([#221](https://github.com/near/cargo-near/pull/221))

### Other

- Gracefully handle missing `git` on `cargo near new` ([#246](https://github.com/near/cargo-near/pull/246))
- update `cargo near new` template `image` and `image_digest` ([#244](https://github.com/near/cargo-near/pull/244))

## [0.11.0](https://github.com/near/cargo-near/compare/cargo-near-v0.10.1...cargo-near-v0.11.0) - 2024-10-29

### Other

- add `passed_env` to default docker template ([#242](https://github.com/near/cargo-near/pull/242))
- update `cargo near new` template `image` and `image_digest` ([#241](https://github.com/near/cargo-near/pull/241))
- cargo near new integration test + gh workflow to autorenew image tag/digest ([#235](https://github.com/near/cargo-near/pull/235))
- [**breaking**] remove unsafe `std::env::set_var` ([#228](https://github.com/near/cargo-near/pull/228))

## [0.10.1](https://github.com/near/cargo-near/compare/cargo-near-v0.10.0...cargo-near-v0.10.1) - 2024-10-17

### Other

- update Docker image reference in new command template with cargo-near 0.10.0 and Rust 1.82.0 ([#232](https://github.com/near/cargo-near/pull/232))

## [0.10.0](https://github.com/near/cargo-near/compare/cargo-near-v0.9.0...cargo-near-v0.10.0) - 2024-10-16

### Added

- [**breaking**] use `wasm-opt -O` (via wasm-opt-rs) as post-step of build ([#231](https://github.com/near/cargo-near/pull/231))
- `env` flag for external parameters of docker build and regular build ([#226](https://github.com/near/cargo-near/pull/226))

### Other

- Use Posthog instead of Mixpanel to collect stats on new projects creation ([#227](https://github.com/near/cargo-near/pull/227))
- Fix tracking usage ([#225](https://github.com/near/cargo-near/pull/225))

## [0.9.0](https://github.com/near/cargo-near/compare/cargo-near-v0.8.2...cargo-near-v0.9.0) - 2024-09-12

### Added

- Fixed GitHub CI staging workflow generated by `cargo near new` command to work correctly with docker case ([#193](https://github.com/near/cargo-near/pull/193))
- Extracted cargo-near-build into a standalone crate to be able to use it in near-workspaces and other places without the rest of the heavy dependencies of cargo-near ([#198](https://github.com/near/cargo-near/pull/198))
- Added tracking of `cargo near new` usage to collect statistics of the command usage ([#192](https://github.com/near/cargo-near/pull/192))

### Fixed

- Addressed warnings in `cargo build -p cargo-near-build` cmd in releaze-plz flow ([#212](https://github.com/near/cargo-near/pull/212))

### Other

- Updated near-* dependencies to 0.26 release ([#220](https://github.com/near/cargo-near/pull/220))
- Use "tracing" for logging and loading indicators ([#216](https://github.com/near/cargo-near/pull/216))
- update docker image and sdk version in `cargo near new` template ([#218](https://github.com/near/cargo-near/pull/218))
- [**breaking**] updates near-* packages to 0.25 version. Updates near-sdk to 5.4 ([#215](https://github.com/near/cargo-near/pull/215))

## [0.8.2](https://github.com/near/cargo-near/compare/cargo-near-v0.8.1...cargo-near-v0.8.2) - 2024-08-16

### Other
- updated near-workspaces-rs ([#205](https://github.com/near/cargo-near/pull/205))

## [0.8.1](https://github.com/near/cargo-near/compare/cargo-near-v0.8.0...cargo-near-v0.8.1) - 2024-08-15

### Other
- update Cargo.lock dependencies

## [0.8.0](https://github.com/near/cargo-near/compare/cargo-near-v0.7.0...cargo-near-v0.8.0) - 2024-08-14

### Other
- [**breaking**] Updated near-* to 0.24, interactive clap to 0.3 ([#201](https://github.com/near/cargo-near/pull/201))
- disable env section of `color_eyre` report ([#196](https://github.com/near/cargo-near/pull/196))

## [0.7.0](https://github.com/near/cargo-near/compare/cargo-near-v0.6.4...cargo-near-v0.7.0) - 2024-08-06

### Added
- Added ability to use SourceScan ([#134](https://github.com/near/cargo-near/pull/134))

### Fixed
- Replacing atty unmaintained dependency ([#194](https://github.com/near/cargo-near/pull/194))

### Other
- update default docker images tags + digests ([#191](https://github.com/near/cargo-near/pull/191))

## [0.6.4](https://github.com/near/cargo-near/compare/cargo-near-v0.6.3...cargo-near-v0.6.4) - 2024-07-22

### Other
- Updated near-sdk and near-workspaces versions in the new project Cargo.toml.template ([#183](https://github.com/near/cargo-near/pull/183))

## [0.6.3](https://github.com/near/cargo-near/compare/cargo-near-v0.6.2...cargo-near-v0.6.3) - 2024-07-03

### Added
- Support passing feature flags to `cargo` invocation ([#160](https://github.com/near/cargo-near/pull/160))

### Fixed
- Also pass feature flags to ABI build step ([#161](https://github.com/near/cargo-near/pull/161))

### Other
- Updates near-cli-rs and cargo-near in the new project template to latest versions ([#168](https://github.com/near/cargo-near/pull/168))
- Updated dependencies to the latest versions ([#167](https://github.com/near/cargo-near/pull/167))
- Updated "interactive_clap" to 0.2.10 (updated "flatten" parameter) ([#154](https://github.com/near/cargo-near/pull/154))

## [0.6.2](https://github.com/near/cargo-near/compare/cargo-near-v0.6.1...cargo-near-v0.6.2) - 2024-04-14

### Added
- Updated new project template with near-sdk-rs 5.1.0 ([#143](https://github.com/near/cargo-near/pull/143))

### Fixed
- Support nixOS - decouple cargo-near from rustup ([#146](https://github.com/near/cargo-near/pull/146))

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
