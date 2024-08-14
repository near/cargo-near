// TODO: make mod non-pub
pub mod fs;
// TODO: make mod non-pub
pub mod cargo_native;
pub mod near;
// TODO: consider making mod non-pub
pub mod pretty_print;
// TODO: make mod non-pub, export `CompilationArtifact` with `pub use`
pub mod types;

// TODO: remove these exports, replace with `pub(crate)` visibility
pub use cargo_native::{ArtifactType, DYLIB, WASM};
