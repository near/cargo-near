use crate::cargo::manifest::CargoManifestPath;
use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, Package};
use std::path::PathBuf;

/// Relevant metadata obtained from Cargo.toml.
#[derive(Debug)]
pub(crate) struct CrateMetadata {
    pub root_package: Package,
    pub target_directory: PathBuf,
    pub manifest_path: CargoManifestPath,
    pub raw_metadata: cargo_metadata::Metadata,
}

impl CrateMetadata {
    /// Parses the contract manifest and returns relevant metadata.
    pub fn collect(manifest_path: CargoManifestPath) -> Result<Self> {
        let (metadata, root_package) = get_cargo_metadata(&manifest_path)?;
        let mut target_directory = metadata.target_directory.as_path().join("near");

        // Normalize the package and lib name.
        let package_name = root_package.name.replace('-', "_");

        let absolute_manifest_dir = manifest_path.directory()?;
        let absolute_workspace_root = metadata.workspace_root.canonicalize()?;
        if absolute_manifest_dir != absolute_workspace_root {
            // If the contract is a package in a workspace, we use the package name
            // as the name of the sub-folder where we put the `.contract` bundle.
            target_directory = target_directory.join(package_name);
        }

        let crate_metadata = CrateMetadata {
            root_package,
            target_directory: target_directory.into(),
            manifest_path,
            raw_metadata: metadata,
        };
        Ok(crate_metadata)
    }
}

/// Get the result of `cargo metadata`, together with the root package id.
fn get_cargo_metadata(
    manifest_path: &CargoManifestPath,
) -> Result<(cargo_metadata::Metadata, Package)> {
    log::info!(
        "Fetching cargo metadata for {}",
        manifest_path.path.to_string_lossy()
    );
    let mut cmd = MetadataCommand::new();
    let metadata = cmd
        .manifest_path(&manifest_path.path)
        .exec()
        .context("Error invoking `cargo metadata`")?;
    let root_package_id = metadata
        .resolve
        .as_ref()
        .and_then(|resolve| resolve.root.as_ref())
        .context("Cannot infer the root project id")?
        .clone();
    // Find the root package by id in the list of packages. It is logical error if the root
    // package is not found in the list.
    let root_package = metadata
        .packages
        .iter()
        .find(|package| package.id == root_package_id)
        .expect("The package is not found in the `cargo metadata` output")
        .clone();
    Ok((metadata, root_package))
}
