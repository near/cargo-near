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
mod abi_exports {
    pub use crate::near::abi::build as build_abi;
    pub use crate::types::near::abi::Opts as AbiOpts;
}

#[cfg(feature = "cli_exports")]
pub use abi_exports::*;

mod build_exports {
    pub use crate::near::build::run as build;
    pub use crate::types::near::build::input::Opts as BuildOpts;
    pub use crate::types::near::build::input::{CliDescription, ColorPreference};
    pub use crate::types::near::build::output::CompilationArtifact as BuildArtifact;
    pub use crate::types::near::build::output::SHA256Checksum;
}
pub use build_exports::*;

mod build_extended_exports {
    pub use crate::near::build_extended::run as build_extended;
    pub use crate::types::near::build_extended::build_script::Opts as BuildScriptOpts;
    pub use crate::types::near::build_extended::OptsExtended as BuildOptsExtended;
}

pub use build_extended_exports::*;

#[cfg(feature = "cli_exports")]
pub use types::near::build::input::BuildContext;

#[cfg(feature = "docker")]
mod docker_build_exports {
    pub use crate::near::docker_build::run as docker_build;
    pub use crate::types::near::docker_build::Opts as DockerBuildOpts;
}

#[cfg(feature = "docker")]
pub use docker_build_exports::*;

pub use camino;
pub use near_abi;
