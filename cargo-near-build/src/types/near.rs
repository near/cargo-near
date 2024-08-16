use std::marker::PhantomData;

use camino::Utf8PathBuf;
use sha2::{Digest, Sha256};

use crate::{ArtifactType, WASM};

// TODO: make non-pub
pub mod abi;

pub struct CompilationArtifact<T: ArtifactType = WASM> {
    pub path: Utf8PathBuf,
    pub fresh: bool,
    pub from_docker: bool,
    pub cargo_near_version_mismatch: VersionMismatch,
    pub artifact_type: PhantomData<T>,
}

impl<T: ArtifactType> CompilationArtifact<T> {
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
// TODO: make non-pub
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

#[derive(Debug)]
pub enum VersionMismatch {
    Some {
        environment: String,
        current_process: String,
    },
    None,
    UnknownFromDocker,
}

impl std::fmt::Display for VersionMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Some {
                environment,
                current_process,
            } => {
                write!(
                    f,
                    "`cargo-near` version {} -> `cargo-near` environment version {}",
                    current_process, environment
                )
            }
            Self::None => write!(f, "no `cargo-near` version mismatch in nested builds detected",),
            Self::UnknownFromDocker => write!(f, "it's unknown if `cargo-near` version mismatch has occurred in docker build environment",),
        }
    }
}
