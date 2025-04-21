#![allow(clippy::needless_lifetimes)]
//! ## Crate features
//!
//! * **build_internal** -
//!   The whole functionality, needed for build and ABI generation, for internal use by `cargo-near`
//!   cli.
//! * **docker** -
//!   Adds `docker` module for functionality of
//!   building in docker with WASM reproducibility.
//!
//! ### Default features
//!
//! None are enabled by default
//!
//! ## Re-exports
//!
//! 1. [camino] is re-exported, because it is used in [BuildOpts], and [BuildArtifact] as type of some of fields
//! 2. [near_abi] is re-exported, because details of ABI generated depends on specific version of `near-abi` dependency  
//! 3. [bon] is re-exported for the convenience of [bon::vec] helper macro
//!
//! ## Sample usage:
//!
//! Default:
//!
//! ```no_run
//! let artifact = cargo_near_build::build(Default::default()).expect("some error during build");
//! ```
//!
//! With some options set:
//!
//! ```no_run
//! let build_opts = cargo_near_build::BuildOpts::builder().features("some_contract_feature_1").build();
//! let artifact = cargo_near_build::build(build_opts).expect("some error during build");
//! ```
pub(crate) mod cargo_native;
/// module contains names of environment variables, exported during
/// various operations of the library
pub mod env_keys;
pub(crate) mod fs;
pub(crate) mod near;
pub(crate) mod pretty_print;
pub(crate) mod types;

#[cfg(feature = "build_internal")]
pub mod abi {
    pub use crate::near::abi::build;
    pub use crate::types::near::abi::Opts as AbiOpts;
}

// TODO #B: extract separate build_exports for `BuildOpts` only and everything it entails
mod build_exports {
    pub use crate::near::build::run as build;
    #[cfg(feature = "docker")]
    pub use crate::types::near::build::input::BuildContext;
    pub use crate::types::near::build::input::Opts as BuildOpts;
    pub use crate::types::near::build::input::{CliDescription, ColorPreference};
    pub use crate::types::near::build::output::CompilationArtifact as BuildArtifact;
    pub use crate::types::near::build::output::SHA256Checksum;
}
pub use build_exports::*;
/// `[cargo_near_build::extended::build]` functionality has been removed for the time being.
///
/// Instead a set of examples how to do a factory build script by running `cargo-near` binary
/// with [std::process::Command] is presented. This approach saves on compilation time when compared
/// to removed `[cargo_near_build::extended::build]`.
///
/// `cargo-near` became a required binary dependency, which needs to be [added](https://github.com/near/cargo-near?tab=readme-ov-file#installation) to build environment and be available in `PATH`.
///
/// - [base example](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-build-script/workspace_root_folder/factory/build.rs#L25-L64)
/// - [adding command flags](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-build-script-with-build-cmd-flags/workspace_root_folder/factory/build.rs#L44-L46)
/// - [realizing logic of passed in environment parameters, not present in source code](https://github.com/dj8yfo/verify_contracts_collection/blob/example-factory-build-script-with-passed-env/workspace_root_folder/factory/build.rs#L26-L37)
pub mod extended {}

#[cfg(feature = "docker")]
pub mod docker {
    pub use crate::near::docker_build::run as build;
    pub use crate::types::near::docker_build::Opts as DockerBuildOpts;
}

#[cfg(feature = "test_code")]
pub use crate::types::cargo::metadata::CrateMetadata;
#[cfg(feature = "test_code")]
pub use crate::types::near::build::buildtime_env::CargoTargetDir;

pub use bon;
pub use camino;
pub use near_abi;
