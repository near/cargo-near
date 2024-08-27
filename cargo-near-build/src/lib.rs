//! ## Re-exports
//!
//! `camino` is re-exported, because it is used in [BuildOpts], and [BuildArtifact]
//! as type of some of fields

pub(crate) mod cargo_native;
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
