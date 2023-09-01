use near_cli_rs::types::path_buf::PathBuf;
use std::path::Path;

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
        self.path.0.parent().ok_or_else(|| {
            anyhow::anyhow!("Unable to infer the directory containing Cargo.toml file")
        })
    }
}

impl TryFrom<PathBuf> for CargoManifestPath {
    type Error = anyhow::Error;

    fn try_from(manifest_path: PathBuf) -> Result<Self, Self::Error> {
        if let Some(file_name) = manifest_path.0.clone().file_name() {
            if file_name != MANIFEST_FILE_NAME {
                anyhow::bail!("the manifest-path must be a path to a Cargo.toml file")
            }
        }

        Ok(CargoManifestPath {
            path: manifest_path,
        })
    }
}
