use crate::types::manifest::CargoManifestPath;
use crate::util;
use camino::Utf8PathBuf;
use cargo_metadata::{MetadataCommand, Package};
use color_eyre::eyre::{ContextCompat, WrapErr};

/// Relevant metadata obtained from Cargo.toml.
#[derive(Debug)]
pub(crate) struct CrateMetadata {
    pub root_package: Package,
    pub target_directory: Utf8PathBuf,
    pub manifest_path: CargoManifestPath,
    pub raw_metadata: cargo_metadata::Metadata,
}

impl CrateMetadata {
    /// Parses the contract manifest and returns relevant metadata.
    pub fn collect(
        manifest_path: CargoManifestPath,
        no_locked: bool,
    ) -> color_eyre::eyre::Result<Self> {
        let (mut metadata, root_package) = get_cargo_metadata(&manifest_path, no_locked)?;

        metadata.target_directory = util::force_canonicalize_dir(&metadata.target_directory)?;
        metadata.workspace_root = metadata.workspace_root.canonicalize_utf8()?;

        let mut target_directory =
            util::force_canonicalize_dir(&metadata.target_directory.join("near"))?;

        // Normalize the package and lib name.
        let package_name = root_package.name.replace('-', "_");

        let absolute_manifest_dir = manifest_path.directory()?;
        if absolute_manifest_dir != metadata.workspace_root {
            // If the contract is a package in a workspace, we use the package name
            // as the name of the sub-folder where we put the `.contract` bundle.
            target_directory = util::force_canonicalize_dir(&target_directory.join(package_name))?;
        }

        let crate_metadata = CrateMetadata {
            root_package,
            target_directory,
            manifest_path,
            raw_metadata: metadata,
        };
        log::trace!("crate metadata : {:#?}", crate_metadata);
        Ok(crate_metadata)
    }

    pub fn resolve_output_dir(
        &self,
        cli_override: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    ) -> color_eyre::eyre::Result<Utf8PathBuf> {
        if let Some(cli_override) = cli_override {
            let out_dir = Utf8PathBuf::from(cli_override);
            return util::force_canonicalize_dir(&out_dir);
        }

        Ok(self.target_directory.clone())
    }
}

/// Get the result of `cargo metadata`, together with the root package id.
fn get_cargo_metadata(
    manifest_path: &CargoManifestPath,
    no_locked: bool,
) -> color_eyre::eyre::Result<(cargo_metadata::Metadata, Package)> {
    log::info!("Fetching cargo metadata for {}", manifest_path.path);
    let mut cmd = MetadataCommand::new();
    if !no_locked {
        cmd.other_options(["--locked".to_string()]);
    }
    let metadata = cmd.manifest_path(&manifest_path.path).exec();
    if let Err(cargo_metadata::Error::CargoMetadata { stderr }) = metadata.as_ref() {
        if stderr.contains("remove the --locked flag") {
            return Err(cargo_metadata::Error::CargoMetadata {
                stderr: stderr.clone(),
            })
            .wrap_err("Cargo.lock is absent or not up-to-date");
        }
    }
    let metadata = metadata
        .wrap_err("Error invoking `cargo metadata`. Your `Cargo.toml` file is likely malformed")?;
    let root_package = metadata
        .root_package()
        .wrap_err("Error invoking `cargo metadata`. Your `Cargo.toml` file is likely malformed")?
        .clone();
    Ok((metadata, root_package))
}
