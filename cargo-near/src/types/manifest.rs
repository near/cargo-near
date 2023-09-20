use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::ContextCompat;

const MANIFEST_FILE_NAME: &str = "Cargo.toml";

/// Path to a `Cargo.toml` file
#[derive(Clone, Debug)]
pub struct CargoManifestPath {
    /// Absolute path to the manifest file
    pub path: Utf8PathBuf,
}

impl CargoManifestPath {
    /// The directory path of the manifest path, if there is one.
    pub fn directory(&self) -> color_eyre::eyre::Result<&Utf8Path> {
        self.path
            .parent()
            .wrap_err("Unable to infer the directory containing Cargo.toml file")
    }
}

impl TryFrom<Utf8PathBuf> for CargoManifestPath {
    type Error = color_eyre::eyre::ErrReport;

    fn try_from(manifest_path: Utf8PathBuf) -> Result<Self, Self::Error> {
        if let Some(file_name) = manifest_path.file_name() {
            if file_name != MANIFEST_FILE_NAME {
                color_eyre::eyre::bail!("the manifest-path must be a path to a Cargo.toml file")
            }
        }
        let canonical_manifest_path = manifest_path.canonicalize_utf8().map_err(|err| match err
            .kind()
        {
            std::io::ErrorKind::NotFound => {
                color_eyre::eyre::eyre!("manifest path `{manifest_path}` does not exist")
            }
            _ => color_eyre::eyre::eyre!("Failed to derive a key from the master key: {err}"),
        })?;
        Ok(CargoManifestPath {
            path: canonical_manifest_path,
        })
    }
}
