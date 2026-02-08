#![allow(clippy::needless_lifetimes)]
//! ## Crate features
//!
//! * **build_external** -
//!   Exports [`crate::build_with_cli`] function which builds contracts by running external `cargo-near` binary
//!   with [`std::process::Command`]
//! * **build_internal** -
//!   The whole functionality, needed for build and ABI generation, mostly for internal use by `cargo-near` CLI implementation
//! * **docker** -
//!   Adds `docker` module for functionality of
//!   building in docker with WASM reproducibility.
//! * **test_code** -
//!   Adds exports needed for integration tests.
//! * **tracing_debug** -
//!   Exports [`crate::init_tracing_debug`] function which initializes tracing with a configuration
//!   that enables all `--teach-me` messages and detailed logging output.
//!
//! ### Default features
//!
//! **build_external**
//!
//! ## Re-exports
//!
//! 1. [`camino`] is re-exported, because it is used in [`BuildOpts`] as type of some of fields
//! 2. [`near_abi`](https://docs.rs/near-abi/latest/near_abi/) is re-exported (under `build_internal` feature), because details of ABI generated depends on specific version of `near-abi` dependency  
//! 3. [`bon`] is re-exported for the convenience of [`bon::vec`] helper macro
//!
//! ## Sample usage:
//!
//! Default:
//!
//! ```no_run
//! let artifact = cargo_near_build::build_with_cli(Default::default()).expect("some error during build");
//! ```
//!
//! With some options set:
//!
//! ```no_run
//! let build_opts = cargo_near_build::BuildOpts::builder().features("some_contract_feature_1").build();
//! let artifact = cargo_near_build::build_with_cli(build_opts).expect("some error during build");
//! ```
#[cfg(any(feature = "build_internal", feature = "docker"))]
pub(crate) mod cargo_native;
/// module contains names of environment variables, exported during
/// various operations of the library
pub mod env_keys;
pub(crate) mod fs;
pub(crate) mod near;
pub(crate) mod pretty_print;
pub(crate) mod types;

#[cfg(feature = "tracing_debug")]
mod tracing_debug;
#[cfg(feature = "tracing_debug")]
pub use tracing_debug::init_tracing_debug;

#[cfg(feature = "build_internal")]
pub mod abi {
    pub use crate::near::abi::build;
    pub use crate::types::near::abi::Opts as AbiOpts;
}

/// these are exports of types, used for both `internal` and `external` build methods
mod build_exports {

    pub use crate::types::near::build::checksum::SHA256Checksum;
    pub use crate::types::near::build::input::Opts as BuildOpts;
    pub use crate::types::near::build::input::{CliDescription, ColorPreference};
}

pub use build_exports::*;

#[cfg(feature = "build_internal")]
pub use crate::near::build::run as build;

#[cfg(any(feature = "build_internal", feature = "docker"))]
pub use crate::types::near::build::output::CompilationArtifact as BuildArtifact;

#[cfg(feature = "build_external")]
pub use crate::near::build_external::run as build_with_cli;
/// Set of examples how to do a factory build script with [`crate::extended::build_with_cli`] api is presented.
///
/// Each example has an equivalent counterpart with no `cargo-near-build` build-dependency, which runs [`std::process::Command`] build with `cargo-near` CLI and has no automatic computation of some
/// of needed build parameters.
///
/// `cargo-near` became a required binary dependency, which needs to be [added](https://github.com/near/cargo-near?tab=readme-ov-file#installation) to build environment and be available in `PATH`.
///
/// - [base example](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-cargo-near-build-extended/workspace_root_folder/factory/build.rs)
///   - [base example: no `cargo-near-build` build-dependency](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-build-script/workspace_root_folder/factory/build.rs#L25-L64)
/// - [adding command flags](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-cargo-near-build-extended-cmd-flags/workspace_root_folder/factory/build.rs#L6-L7)
///   - [adding command flags: no `cargo-near-build` build-dependency](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-build-script-with-build-cmd-flags/workspace_root_folder/factory/build.rs#L44-L46)
/// - [passed in environment parameters, not present in source code](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-cargo-near-build-extended-passed-env/workspace_root_folder/factory/build.rs#L11)
///   - This is only relevant in reproducible builds context, where host env doesn't populate into build by default.
///   - It depends on also [passing the same env](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-cargo-near-build-extended-passed-env/workspace_root_folder/factory/Cargo.toml#L24) to the contract itself, otherwise these vars won't propagate to sub-contract.
///   - [passed in environment parameters, not present in source code: no `cargo-near-build` build-dependency](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-build-script-with-passed-env/workspace_root_folder/factory/build.rs#L26-L37)
///   
#[cfg(feature = "build_external")]
pub mod extended {
    pub use crate::near::build_rs_build_external::run as build_with_cli;
    pub use crate::types::near::build_extended::input::*;
}

#[cfg(feature = "docker")]
pub mod docker {
    pub use crate::near::docker_build::run as build;
    pub use crate::types::near::build::input::BuildContext;
    pub use crate::types::near::docker_build::Opts as DockerBuildOpts;
}

#[cfg(feature = "test_code")]
pub use crate::types::cargo::metadata::CrateMetadata;
#[cfg(feature = "test_code")]
pub use crate::types::near::build::common_buildtime_env::CargoTargetDir;

pub use bon;
pub use camino;
#[cfg(feature = "build_internal")]
pub use near_abi;
