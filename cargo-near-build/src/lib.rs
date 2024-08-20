// TODO: make mod non-pub
pub mod fs;
// TODO: make mod non-pub
pub mod cargo_native;
// TODO: make mod non-pub
pub mod env_keys;
pub mod near;
// TODO: consider making mod non-pub
pub mod pretty_print;
// TODO: make mod non-pub, export `CompilationArtifact` and `VersionMismatch` with `pub use`
pub mod types;

// TODO: remove these exports, replace with `pub(crate)` visibility
pub use cargo_native::{ArtifactType, DYLIB, WASM};

pub use near::abi::build as build_abi;
// used in `AbiOpts` and `BuildOpts`
pub use types::color_preference::ColorPreference;
pub use types::near::abi::Opts as AbiOpts;
pub use types::near::build::Opts as BuildOpts;

pub use types::near::build::version_mismatch::VersionMismatch;
pub use types::near::build::CompilationArtifact as BuildArtifact;

pub use near_abi;
// used in `AbiOpts` and `BuildOpts`, and `BuildArtifact`
pub use camino;
