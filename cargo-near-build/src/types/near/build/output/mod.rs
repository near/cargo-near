use std::marker::PhantomData;

use crate::cargo_native::{ArtifactType, Wasm};
use camino::Utf8PathBuf;
use sha2::{Digest, Sha256};
pub mod version_mismatch;

/// type of success value of result of [crate::build]
pub struct CompilationArtifact<T: ArtifactType = Wasm> {
    /// path to output file
    pub path: Utf8PathBuf,
    pub fresh: bool,
    /// whether the artifact file originated from docker build or regular build with rust toolchain
    pub from_docker: bool,
    pub(crate) builder_version_mismatch: version_mismatch::VersionMismatch,
    pub(crate) artifact_type: PhantomData<T>,
}

impl crate::BuildArtifact {
    pub fn compute_hash(&self) -> eyre::Result<SHA256Checksum> {
        let mut hasher = Sha256::new();
        hasher.update(std::fs::read(&self.path)?);
        let hash = hasher.finalize();
        let hash: &[u8] = hash.as_ref();
        Ok(SHA256Checksum {
            hash: hash.to_vec(),
        })
    }
}

/// type of return value of [crate::BuildArtifact::compute_hash]
pub struct SHA256Checksum {
    pub hash: Vec<u8>,
}

impl SHA256Checksum {
    pub fn to_hex_string(&self) -> String {
        hex::encode(&self.hash)
    }

    pub fn to_base58_string(&self) -> String {
        bs58::encode(&self.hash).into_string()
    }
}
