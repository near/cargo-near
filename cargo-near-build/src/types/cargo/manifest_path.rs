use camino::{Utf8Path, Utf8PathBuf};
use eyre::ContextCompat;

pub const MANIFEST_FILE_NAME: &str = "Cargo.toml";

/// Path to a `Cargo.toml` file
#[derive(Clone, Debug)]
pub struct ManifestPath {
    /// Absolute path to the manifest file
    pub path: Utf8PathBuf,
}

impl ManifestPath {
    /// The directory path of the manifest path, if there is one.
    pub fn directory(&self) -> eyre::Result<&Utf8Path> {
        self.path
            .parent()
            .wrap_err("Unable to infer the directory containing Cargo.toml file")
    }
}

impl TryFrom<Utf8PathBuf> for ManifestPath {
    type Error = eyre::ErrReport;

    fn try_from(manifest_path: Utf8PathBuf) -> Result<Self, Self::Error> {
        match manifest_path.file_name() {
            None => {
                eyre::bail!("the manifest-path must be a path to a Cargo.toml file")
            }
            Some(file_name) if file_name != MANIFEST_FILE_NAME => {
                eyre::bail!("the manifest-path must be a path to a Cargo.toml file")
            }
            _ => {}
        }
        let canonical_manifest_path = manifest_path.canonicalize_utf8().map_err(|err| match err
            .kind()
        {
            std::io::ErrorKind::NotFound => {
                match std::env::current_dir() {
                    Ok(pwd ) => {
                        let pwd = pwd.to_string_lossy();
                        eyre::eyre!("manifest path `{manifest_path}` in `{pwd}` does not exist")
                    },
                    Err(err) => {
                        eyre::eyre!("manifest path `{manifest_path}` in `workdir not determined: {:?}` does not exist",
                            err
                        )
                    }
                }
            }
            _ => eyre::eyre!("manifest_path.canonicalize_utf8() error: {err}"),
        })?;
        Ok(ManifestPath {
            path: canonical_manifest_path,
        })
    }
}
