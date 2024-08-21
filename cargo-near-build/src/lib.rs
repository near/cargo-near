pub(crate) mod cargo_native;
pub mod env_keys;
pub(crate) mod fs;
pub(crate) mod near;
// TODO: replace `pub` with `pub(crate)` on docker logic moved
pub mod pretty_print;
pub(crate) mod types;

// TODO: remove on docker logic moved
pub use types::cargo::manifest_path::{ManifestPath, MANIFEST_FILE_NAME};
// TODO: remove on docker logic moved
pub use types::cargo::metadata::CrateMetadata;
// TODO: remove on docker logic moved
pub use types::near::build::ArtifactMessages;

// used in `AbiOpts` and `BuildOpts`
pub use types::color_preference::ColorPreference;
pub use types::near::abi::Opts as AbiOpts;
pub use types::near::build::{CliDescription, Opts as BuildOpts};

pub use types::near::build::version_mismatch::VersionMismatch;
pub use types::near::build::CompilationArtifact as BuildArtifact;
pub use types::near::build::SHA256Checksum;
pub use types::near::build_extended::{
    build_script::Opts as BuildScriptOpts, OptsExtended as BuildOptsExtended,
};

pub use near_abi;
// used in `AbiOpts` and `BuildOpts`, and `BuildArtifact`
pub use camino;

pub use near::abi::build as build_abi;
pub use near::build::run as build;
pub use near::build_extended::run as build_extended;
