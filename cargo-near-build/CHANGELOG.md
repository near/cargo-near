# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0](https://github.com/near/cargo-near/compare/cargo-near-build-v0.7.2...cargo-near-build-v0.8.0) - 2025-08-31

### Added

- Added instructions for users on compiling the project on unsupported versions of Rust. ([#357](https://github.com/near/cargo-near/pull/357))

## [0.7.2](https://github.com/near/cargo-near/compare/cargo-near-build-v0.7.1...cargo-near-build-v0.7.2) - 2025-07-08

### Other

- Fixed linting errors - non-inlined formatting syntax ([#354](https://github.com/near/cargo-near/pull/354))

## [0.7.1](https://github.com/near/cargo-near/compare/cargo-near-build-v0.7.0...cargo-near-build-v0.7.1) - 2025-05-19

### Added

- add `Feature::TruncSat` and `Feature::BulkMemory` to `wasm_opt::OptimizationOptions` ([#338](https://github.com/near/cargo-near/pull/338))
- `--variant <name>` flag ([#339](https://github.com/near/cargo-near/pull/339))

## [0.7.0](https://github.com/near/cargo-near/compare/cargo-near-build-v0.6.0...cargo-near-build-v0.7.0) - 2025-05-16

### Other

- use 1.86.0 toolchain for contracts tests with live node ([#340](https://github.com/near/cargo-near/pull/340))

## [0.6.0](https://github.com/near/cargo-near/compare/cargo-near-build-v0.5.1...cargo-near-build-v0.6.0) - 2025-05-03

### Added

- [cargo_near_build::extended::build_with_cli] for build.rs of factories ([#334](https://github.com/near/cargo-near/pull/334))
- [cargo_near_build::build_with_cli] method in `build_external` default feature ([#333](https://github.com/near/cargo-near/pull/333))

## [0.5.1](https://github.com/near/cargo-near/compare/cargo-near-build-v0.5.0...cargo-near-build-v0.5.1) - 2025-04-21

### Fixed

- doc publish. Change reference to `latest` instead of specific version ([#329](https://github.com/near/cargo-near/pull/329))

## [0.5.0](https://github.com/near/cargo-near/compare/cargo-near-build-v0.4.6...cargo-near-build-v0.5.0) - 2025-04-21

### Added

- [**breaking**] pass in external nep330_wasm_output path from env as override ([#328](https://github.com/near/cargo-near/pull/328))
- populate `output_wasm_path` into `ContractSourceMetadata` ([#323](https://github.com/near/cargo-near/pull/323))

## [0.4.6](https://github.com/near/cargo-near/compare/cargo-near-build-v0.4.5...cargo-near-build-v0.4.6) - 2025-04-08

### Added

- extract `near-verify-rs` dependency ([#320](https://github.com/near/cargo-near/pull/320))

## [0.4.5](https://github.com/near/cargo-near/compare/cargo-near-build-v0.4.4...cargo-near-build-v0.4.5) - 2025-03-20

### Other

- fix clippy 1.85 ([#311](https://github.com/near/cargo-near/pull/311))

## [0.4.4](https://github.com/near/cargo-near/compare/cargo-near-build-v0.4.3...cargo-near-build-v0.4.4) - 2025-02-13

### Added

- embed docs for flags/arguments for `-h`/`--help` (#304)

## [0.4.3](https://github.com/near/cargo-near/compare/cargo-near-build-v0.4.2...cargo-near-build-v0.4.3) - 2025-01-22

### Fixed

- remove from env CARGO_ENCODED_RUSTFLAGS for easier nested builds, simplify RUSTFLAGS computation rule (#289)

### Other

- optimize out retriggering 2nd stage of build + of wasmopt stage in tests context (#292)
- update `cargo near new` template `image` and `image_digest` ([#288](https://github.com/near/cargo-near/pull/288))
- unpin `cc` after issue resolution (#285)

## [0.4.2](https://github.com/near/cargo-near/compare/cargo-near-build-v0.4.1...cargo-near-build-v0.4.2) - 2024-12-19

### Other

- update deps , pin `cc` (#282)

## [0.4.1](https://github.com/near/cargo-near/compare/cargo-near-build-v0.4.0...cargo-near-build-v0.4.1) - 2024-12-18

### Fixed

- running `near_workspaces::compile_project` concurrently in tests (#266)

## [0.4.0](https://github.com/near/cargo-near/compare/cargo-near-build-v0.3.2...cargo-near-build-v0.4.0) - 2024-12-17

### Added

- reproducible choice interactive (#262)

### Other

- fix 1.83clippy, audit (#260)

## [0.3.2](https://github.com/near/cargo-near/compare/cargo-near-build-v0.3.1...cargo-near-build-v0.3.2) - 2024-11-19

### Fixed

- Replaced --teach-me printing from docker_args to docker_cmd ([#248](https://github.com/near/cargo-near/pull/248))

## [0.3.1](https://github.com/near/cargo-near/compare/cargo-near-build-v0.3.0...cargo-near-build-v0.3.1) - 2024-11-14

### Added

- Added the ability to use the TEACH-ME mode ([#221](https://github.com/near/cargo-near/pull/221))

## [0.3.0](https://github.com/near/cargo-near/compare/cargo-near-build-v0.2.0...cargo-near-build-v0.3.0) - 2024-10-29

### Other

- cargo near new integration test + gh workflow to autorenew image tag/digest ([#235](https://github.com/near/cargo-near/pull/235))
- [**breaking**] remove unsafe `std::env::set_var` ([#228](https://github.com/near/cargo-near/pull/228))

## [0.2.0](https://github.com/near/cargo-near/compare/cargo-near-build-v0.1.1...cargo-near-build-v0.2.0) - 2024-10-16

### Added

- [**breaking**] use `wasm-opt -O` (via wasm-opt-rs) as post-step of build ([#231](https://github.com/near/cargo-near/pull/231))
- `env` flag for external parameters of docker build and regular build ([#226](https://github.com/near/cargo-near/pull/226))

### Other

- disable github release for `cargo-near-build` via cargo-dist ([#222](https://github.com/near/cargo-near/pull/222))

## [0.1.1](https://github.com/near/cargo-near/compare/cargo-near-build-v0.1.0...cargo-near-build-v0.1.1) - 2024-09-12

### Added

- Fixed GitHub CI staging workflow generated by `cargo near new` command to work correctly with docker case ([#193](https://github.com/near/cargo-near/pull/193))

### Other

- Use "tracing" for logging and loading indicators ([#216](https://github.com/near/cargo-near/pull/216))
- fix clippy ([#219](https://github.com/near/cargo-near/pull/219))
