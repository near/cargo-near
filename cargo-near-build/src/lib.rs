pub(crate) mod cargo_native;
pub mod env_keys;
pub(crate) mod fs;
pub(crate) mod near;
pub(crate) mod pretty_print;
pub(crate) mod types;

pub use types::near::abi::Opts as AbiOpts;
pub use types::near::build::input::BuildContext;
pub use types::near::build::input::ColorPreference;
pub use types::near::build::input::{CliDescription, Opts as BuildOpts};
#[cfg(feature = "docker")]
pub use types::near::docker_build::Opts as DockerBuildOpts;

pub use types::near::build::output::CompilationArtifact as BuildArtifact;
pub use types::near::build::output::SHA256Checksum;
pub use types::near::build_extended::{
    build_script::Opts as BuildScriptOpts, OptsExtended as BuildOptsExtended,
};

pub use near_abi;
// used in `AbiOpts` and `BuildOpts`, and `BuildArtifact`
pub use camino;

pub use near::abi::build as build_abi;
pub use near::build::run as build;
pub use near::build_extended::run as build_extended;
#[cfg(feature = "docker")]
pub use near::docker_build::run as docker_build;
