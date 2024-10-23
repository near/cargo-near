use std::{thread, time::Duration};

use camino::Utf8PathBuf;
use cargo_metadata::{MetadataCommand, Package};
use colored::Colorize;
use eyre::{ContextCompat, WrapErr};

use crate::types::near::build::buildtime_env;

use super::manifest_path::ManifestPath;

/// Relevant metadata obtained from Cargo.toml.
#[derive(Debug)]
pub struct CrateMetadata {
    pub root_package: Package,
    pub target_directory: Utf8PathBuf,
    pub manifest_path: ManifestPath,
    pub raw_metadata: cargo_metadata::Metadata,
}

impl CrateMetadata {
    /// Parses the contract manifest and returns relevant metadata.
    pub fn collect(
        manifest_path: ManifestPath,
        no_locked: bool,
        cargo_target_dir: Option<&buildtime_env::CargoTargetDir>,
    ) -> eyre::Result<Self> {
        let (mut metadata, root_package) =
            get_cargo_metadata(&manifest_path, no_locked, cargo_target_dir)?;

        metadata.target_directory = crate::fs::force_canonicalize_dir(&metadata.target_directory)?;
        metadata.workspace_root = metadata.workspace_root.canonicalize_utf8()?;

        let mut target_directory =
            crate::fs::force_canonicalize_dir(&metadata.target_directory.join("near"))?;

        // Normalize the package and lib name.
        let package_name = root_package.name.replace('-', "_");

        let absolute_manifest_dir = manifest_path.directory()?;
        if absolute_manifest_dir != metadata.workspace_root {
            // If the contract is a package in a workspace, we use the package name
            // as the name of the sub-folder where we put the `.contract` bundle.
            target_directory =
                crate::fs::force_canonicalize_dir(&target_directory.join(package_name))?;
        }

        let crate_metadata = CrateMetadata {
            root_package,
            target_directory,
            manifest_path,
            raw_metadata: metadata,
        };
        tracing::trace!("crate metadata : {:#?}", crate_metadata);
        Ok(crate_metadata)
    }

    pub fn resolve_output_dir(
        &self,
        cli_override: Option<Utf8PathBuf>,
    ) -> eyre::Result<Utf8PathBuf> {
        let result = if let Some(cli_override) = cli_override {
            crate::fs::force_canonicalize_dir(&cli_override)?
        } else {
            self.target_directory.clone()
        };
        tracing::debug!("resolved output directory: {}", result);
        Ok(result)
    }

    pub fn formatted_package_name(&self) -> String {
        self.root_package.name.replace('-', "_")
    }

    pub fn find_dependency(&self, dependency_name: &str) -> eyre::Result<&cargo_metadata::Package> {
        let root_node = self
            .raw_metadata
            .resolve
            .as_ref()
            .and_then(|dep_graph| {
                dep_graph
                .nodes
                .iter()
                .find(|node| node.id == self.root_package.id)
            })
            .wrap_err("unable to appropriately resolve the dependency graph, perhaps your `Cargo.toml` file is malformed")?;

        root_node
            .deps
            .iter()
            .find(|dep| dep.name == dependency_name)
            .and_then(|near_sdk| {
                self.raw_metadata
                    .packages
                    .iter()
                    .find(|pkg| pkg.id == near_sdk.pkg)
            })
            .wrap_err(format!("`{}` dependency not found", dependency_name))
    }
}

/// Get the result of `cargo metadata`, together with the root package id.
fn get_cargo_metadata(
    manifest_path: &ManifestPath,
    no_locked: bool,
    cargo_target_dir: Option<&buildtime_env::CargoTargetDir>,
) -> eyre::Result<(cargo_metadata::Metadata, Package)> {
    tracing::info!("Fetching cargo metadata for {}", manifest_path.path);
    let mut cmd = MetadataCommand::new();
    if !no_locked {
        cmd.other_options(["--locked".to_string()]);
    }
    if let Some(target_dir) = cargo_target_dir {
        let (key, value) = target_dir.entry();
        cmd.env(key, value);
    }
    let cmd = cmd.manifest_path(&manifest_path.path);
    tracing::debug!("metadata command: {:#?}", cmd.cargo_command());
    let metadata = cmd.exec();
    if let Err(cargo_metadata::Error::CargoMetadata { stderr }) = metadata.as_ref() {
        if stderr.contains("remove the --locked flag") {
            println!(
                "{}",
                "An error with Cargo.lock has been encountered...".yellow()
            );
            println!(
                "{}",
                "You can choose to disable `--locked` flag for downstream `cargo` command with `--no-locked` flag.".cyan()
            );
            thread::sleep(Duration::new(5, 0));
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
