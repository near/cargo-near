use std::marker::PhantomData;

use crate::{
    cargo_native::{ArtifactType, Wasm},
    SHA256Checksum,
};
use camino::Utf8PathBuf;
pub mod version_info;

/// type of success value of result of [build](crate::build) function
pub struct CompilationArtifact<T: ArtifactType = Wasm> {
    /// path to output file
    pub path: Utf8PathBuf,
    pub fresh: bool,
    /// whether the artifact file originated from docker build or regular build with rust toolchain
    pub from_docker: bool,
    /// `None` of `Option` means it hasn't been set yet
    #[allow(unused)]
    pub(crate) builder_version_info: Option<version_info::VersionInfo>,
    pub(crate) artifact_type: PhantomData<T>,
}

impl crate::BuildArtifact {
    pub fn compute_hash(&self) -> eyre::Result<SHA256Checksum> {
        SHA256Checksum::new(&self.path)
    }
}
