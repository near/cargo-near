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
//! let artifact = cargo_near_build::build(Default::default(), None).expect("some error during build");
//! ```
//!
//! With some options set:
//!
//! ```no_run
//! let build_opts = cargo_near_build::BuildOpts::builder().features("some_contract_feature_1").build();
//! let artifact = cargo_near_build::build(build_opts, None).expect("some error during build");
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
    pub use crate::types::near::build::input::implicit_env::Opts as BuildImplicitEnvOpts;
    #[cfg(feature = "docker")]
    pub use crate::types::near::build::input::BuildContext;
    pub use crate::types::near::build::input::Opts as BuildOpts;
    pub use crate::types::near::build::input::{CliDescription, ColorPreference};
    pub use crate::types::near::build::output::CompilationArtifact as BuildArtifact;
    pub use crate::types::near::build::output::SHA256Checksum;
}
pub use build_exports::*;

/// module is available if crate is built with `features = ["build_script"]`.
///
/// Contains an extended `build` method used to build contracts, that current crate
/// depends on, in `build.rs` of current crate
/// Potential import may look like this:
/// ```ignore
/// [build-dependencies.cargo-near-build]
/// version = "x.y.z"
/// features = ["build_script"]
/// ```
///
/// Usage example:
///
/// ```no_run
/// use cargo_near_build::{bon, extended};
/// use cargo_near_build::{BuildImplicitEnvOpts, BuildOpts};
///
/// // directory of target sub-contract's crate
/// let workdir = "../another-contract";
/// // unix path to target sub-contract's crate from root of the repo
/// let nep330_contract_path = "another-contract";
///
/// let build_opts = BuildOpts::builder().build(); // default opts
///
/// let pwd = std::env::current_dir().expect("get pwd");
/// // a distinct target is needed to avoid deadlock during build
/// let distinct_target = pwd.join("../target/build-rs-another-contract");
/// let stub_path = pwd.join("../target/stub.bin");
///
/// let build_implicit_env_opts = BuildImplicitEnvOpts::builder()
///     .nep330_contract_path(nep330_contract_path)
///     .cargo_target_dir(distinct_target.to_string_lossy())
///     .build();
///
/// let build_script_opts = extended::BuildScriptOpts::builder()
///     .rerun_if_changed_list(bon::vec![workdir, "../Cargo.toml", "../Cargo.lock",])
///     .build_skipped_when_env_is(vec![
///         // shorter build for `cargo check`
///         ("PROFILE", "debug"),
///         (cargo_near_build::env_keys::BUILD_RS_ABI_STEP_HINT, "true"),
///     ])
///     .stub_path(stub_path.to_string_lossy())
///     .result_env_key("BUILD_RS_SUB_BUILD_ARTIFACT_1")
///     .build();
///
/// let extended_opts = extended::BuildOptsExtended::builder()
///     .workdir(workdir)
///     .build_opts(build_opts)
///     .build_implicit_env_opts(build_implicit_env_opts)
///     .build_script_opts(build_script_opts)
///     .build();
///
/// cargo_near_build::extended::build(extended_opts).expect("sub-contract build error");
/// ```
#[cfg(feature = "build_script")]
pub mod extended {
    pub use crate::near::build_extended::run as build;
    pub use crate::types::near::build_extended::build_script::{EnvPairs, Opts as BuildScriptOpts};
    pub use crate::types::near::build_extended::OptsExtended as BuildOptsExtended;
}

#[cfg(feature = "docker")]
pub mod docker {
    pub use crate::near::docker_build::run as build;
    pub use crate::types::near::docker_build::Opts as DockerBuildOpts;
}

pub use bon;
pub use camino;
pub use near_abi;
