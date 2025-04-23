#[cfg(feature = "build_internal")]
pub mod buildtime_env;
/// these are env_variables, used both for `build_internal`,
/// and `build_external` features
pub mod common_buildtime_env;
pub mod input;
#[cfg(any(feature = "build_internal", feature = "docker"))]
pub mod output;
#[cfg(any(feature = "build_internal", feature = "docker"))]
pub mod side_effects;
pub mod checksum {
    use camino::Utf8PathBuf;
    use sha2::{Digest, Sha256};

    /// convenience helper to compute resulting artifact hashsum if needed
    pub struct SHA256Checksum {
        pub hash: Vec<u8>,
    }

    impl SHA256Checksum {
        pub fn new(file: &Utf8PathBuf) -> eyre::Result<SHA256Checksum> {
            let mut hasher = Sha256::new();
            hasher.update(std::fs::read(file)?);
            let hash = hasher.finalize();
            let hash: &[u8] = hash.as_ref();
            Ok(SHA256Checksum {
                hash: hash.to_vec(),
            })
        }
        pub fn to_hex_string(&self) -> String {
            hex::encode(&self.hash)
        }

        pub fn to_base58_string(&self) -> String {
            bs58::encode(&self.hash).into_string()
        }
    }
}
