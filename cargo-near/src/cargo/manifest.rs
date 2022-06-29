use std::path::{Path, PathBuf};

const MANIFEST_FILE_NAME: &str = "Cargo.toml";

/// Path to a `Cargo.toml` file
#[derive(Clone, Debug)]
pub struct CargoManifestPath {
    /// Absolute path to the manifest file
    pub path: PathBuf,
}

impl CargoManifestPath {
    /// The directory path of the manifest path, if there is one.
    pub fn directory(&self) -> anyhow::Result<&Path> {
        self.path.parent().ok_or_else(|| {
            anyhow::anyhow!("Unable to infer the directory containing Cargo.toml file")
        })
    }
}

impl TryFrom<PathBuf> for CargoManifestPath {
    type Error = anyhow::Error;

    fn try_from(manifest_path: PathBuf) -> Result<Self, Self::Error> {
        if let Some(file_name) = manifest_path.file_name() {
            if file_name != MANIFEST_FILE_NAME {
                anyhow::bail!("Manifest file must be a Cargo.toml")
            }
        }
        let canonical_manifest_path = manifest_path.canonicalize().map_err(|err| {
            anyhow::anyhow!("Failed to canonicalize {:?}: {:?}", manifest_path, err)
        })?;
        Ok(CargoManifestPath {
            path: canonical_manifest_path,
        })
    }
}
