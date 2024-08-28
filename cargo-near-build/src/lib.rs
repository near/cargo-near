//! ## Re-exports
//!
//! 1. [camino] is re-exported, because it is used in [BuildOpts], and [BuildArtifact]
//! as type of some of fields
//! 2. [near_abi] is re-exported, because details of ABI generated depends on specific version of `near-abi` dependency  
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
//!     let build_opts = cargo_near_build::BuildOpts {
//!         features: Some("some-contract-feature-1".into()),
//!         ..Default::default()
//!     };
//!     let artifact = cargo_near_build::build(build_opts).expect("some error during build");
//! ```
pub(crate) mod cargo_native;
/// the module contains names of environment variables, exported during
/// various operations of the library
pub mod env_keys;
pub(crate) mod fs;
pub(crate) mod near;
pub(crate) mod pretty_print;
pub(crate) mod types;

#[cfg(feature = "cli_exports")]
pub mod abi {
    pub use crate::near::abi::build;
    pub use crate::types::near::abi::Opts as AbiOpts;
}

mod build_exports {
    pub use crate::near::build::run as build;
    #[cfg(feature = "cli_exports")]
    pub use crate::types::near::build::input::BuildContext;
    pub use crate::types::near::build::input::Opts as BuildOpts;
    pub use crate::types::near::build::input::{CliDescription, ColorPreference};
    pub use crate::types::near::build::output::CompilationArtifact as BuildArtifact;
    pub use crate::types::near::build::output::SHA256Checksum;
}
pub use build_exports::*;

/// Module is available if crate is built with `features = ["build_script"]`.
///
/// Contains an extended `build` method used to build contracts, that current crate
/// depends on, in `build.rs` of current crate
/// Potential import may look like this:
/// ```ignore
/// [build-dependencies.cargo-near-build]
/// version = "0.1.0"
/// features = ["build_script"]
/// ```
///
/// Usage example:
///
/// ```no_run
/// use cargo_near_build::extended::BuildScriptOpts;
/// let opts = cargo_near_build::extended::BuildOptsExtended {
///     workdir: "../another-contract",
///     env: vec![
///         // unix path of target contract from root of repo
///         (cargo_near_build::env_keys::nep330::CONTRACT_PATH, "another-contract")
///     ],
///     build_opts: cargo_near_build::BuildOpts::default(),
///     build_script_opts: BuildScriptOpts {
///         result_env_key: Some("BUILD_RS_SUB_BUILD_ARTIFACT_1"),
///         rerun_if_changed_list: vec!["../another-contract", "../Cargo.toml", "../Cargo.lock"],
///         build_skipped_when_env_is: vec![
///             // shorter build for `cargo check`
///             ("PROFILE", "debug"),
///             (cargo_near_build::env_keys::BUILD_RS_ABI_STEP_HINT, "true"),
///         ],
///         distinct_target_dir: Some("../target/build-rs-another-contract"),
///         stub_path: Some("../target/stub.bin"),
///     },
/// };
/// cargo_near_build::extended::build(opts).expect("sub-contract build error");
/// ```
#[cfg(feature = "build_script")]
pub mod extended {
    pub use crate::near::build_extended::run as build;
    pub use crate::types::near::build_extended::build_script::Opts as BuildScriptOpts;
    pub use crate::types::near::build_extended::OptsExtended as BuildOptsExtended;
}

#[cfg(feature = "docker")]
pub mod docker {
    pub use crate::near::docker_build::run as build;
    pub use crate::types::near::docker_build::Opts as DockerBuildOpts;
}

pub use camino;
pub use near_abi;
