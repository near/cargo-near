use std::marker::PhantomData;

use crate::cargo_native::{ArtifactType, Wasm};
use camino::Utf8PathBuf;
use sha2::{Digest, Sha256};
pub mod version_mismatch;

pub struct CompilationArtifact<T: ArtifactType = Wasm> {
    pub path: Utf8PathBuf,
    pub fresh: bool,
    pub from_docker: bool,
    // TODO: make pub(crate)
    pub builder_version_mismatch: version_mismatch::VersionMismatch,
    // TODO: make pub(crate)
    pub artifact_type: PhantomData<T>,
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
