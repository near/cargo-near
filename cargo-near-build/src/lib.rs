#![allow(clippy::needless_lifetimes)]
//! ## Crate features
//!
//! * **build_script** -
//!   Adds [extended] module for use in build scripts
//! * **abi_build** -
//!   Additional functionality, needed for build of ABI separately
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

#[cfg(feature = "abi_build")]
pub mod abi {
    pub use crate::near::abi::build;
    pub use crate::types::near::abi::Opts as AbiOpts;
}

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
