use std::{thread, time::Duration};

use camino::Utf8PathBuf;
#[cfg(any(feature = "test_code", feature = "docker"))]
use cargo_metadata::DepKindInfo;

use cargo_metadata::{MetadataCommand, Package};
use colored::Colorize;
#[cfg(any(feature = "test_code", feature = "docker"))]
use eyre::OptionExt;
use eyre::{ContextCompat, WrapErr};

use crate::pretty_print;
use crate::types::near::build::common_buildtime_env;
use crate::types::near::OutputPaths;

use super::manifest_path::ManifestPath;

/// Relevant metadata obtained from Cargo.toml.
#[derive(Debug)]
// TODO #F: uncomment for `build_external_extended` method
#[allow(unused)]
pub struct CrateMetadata {
    pub root_package: Package,
    pub target_directory: Utf8PathBuf,
    pub manifest_path: ManifestPath,
    pub raw_metadata: cargo_metadata::Metadata,
}

impl CrateMetadata {
    /// Parses the contract manifest and returns relevant metadata.
    // TODO #F: uncomment for `build_external_extended` method
    #[allow(unused)]
    pub fn collect(
        manifest_path: ManifestPath,
        no_locked: bool,
        cargo_target_dir: &common_buildtime_env::CargoTargetDir,
    ) -> eyre::Result<Self> {
        let (metadata, root_package) = {
            let (mut metadata, root_package) =
                get_cargo_metadata(&manifest_path, no_locked, cargo_target_dir)?;
            metadata.target_directory =
                crate::fs::force_canonicalize_dir(&metadata.target_directory)?;
            metadata.workspace_root = metadata.workspace_root.canonicalize_utf8()?;
            (metadata, root_package)
        };

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

    pub(in crate::types) fn resolve_output_dir(
        &self,
        cli_override: Option<Utf8PathBuf>,
    ) -> eyre::Result<Utf8PathBuf> {
        let result = if let Some(cli_override) = cli_override {
            crate::fs::force_canonicalize_dir(&cli_override)?
        } else {
            self.target_directory.clone()
        };
        tracing::info!(
            target: "near_teach_me",
            parent: &tracing::Span::none(),
            "Resolved output directory: {}", result
        );
        assert!(
            result.is_absolute(),
            "{result} expected to be an absolute path"
        );
        Ok(result)
    }

    pub fn formatted_package_name(&self) -> String {
        self.root_package.name.replace('-', "_")
    }
    /// NOTE important!: the way the output path for wasm file is resolved now cannot change,
    /// as the implementation in contracts' verification will continue to compute output path
    /// according to https://github.com/near/near-verify-rs/blob/aba996522d99d26c7212961504ab40807a4d59fe/src/types/internal/legacy_rust/metadata.rs#L73-L79
    ///
    /// and implementation of initial docker build also assumes the same destination
    // TODO #F: uncomment for `build_external_extended` method
    #[allow(unused)]
    pub fn get_legacy_cargo_near_output_path(
        &self,
        cli_override: Option<Utf8PathBuf>,
    ) -> eyre::Result<OutputPaths> {
        OutputPaths::new(self, cli_override)
    }

    #[cfg(any(feature = "test_code", feature = "docker"))]
    pub fn find_direct_dependency(
        &self,
        dependency_name: &str,
    ) -> eyre::Result<Vec<(&cargo_metadata::Package, Vec<DepKindInfo>)>> {
        let Some(ref dependency_graph) = self.raw_metadata.resolve else {
            return Err(eyre::eyre!(
                "crate_metadata.raw_metadata.resolve dependency graph is expected to be set\n\
                it's not set when `cargo metadata` was run with `--no-deps` flag"
            ));
        };
        let Some(ref root_package_id) = dependency_graph.root else {
            return Err(eyre::eyre!(
                "crate_metadata.raw_metadata.resolve.root package id is expected to be set\n\
                it's not set when `cargo metadata` was run from a root of virtual workspace"
            ));
        };

        let root_nodes = dependency_graph
            .nodes
            .iter()
            .filter(|node| node.id == *root_package_id)
            .collect::<Vec<_>>();

        if root_nodes.len() != 1 {
            return Err(eyre::eyre!(
                "expected to find exactly 1 root node in dependency graph: {:#?}",
                root_nodes
            ));
        }
        let root_node = root_nodes[0];

        let dependency_nodes = root_node
            .deps
            .iter()
            .filter(|dep| dep.name == dependency_name)
            .collect::<Vec<_>>();

        let mut result = vec![];

        for dependency_node in dependency_nodes {
            let dependency_package = self
                .raw_metadata
                .packages
                .iter()
                .find(|pkg| pkg.id == dependency_node.pkg)
                .ok_or_eyre(format!(
                    "expected to find a package for package id : {:#?}",
                    dependency_node.pkg
                ))?;
            result.push((dependency_package, dependency_node.dep_kinds.clone()));
        }
        Ok(result)
    }
}

/// Runs configured `cargo metadata` and returns parsed `Metadata`.
/// this is copy-pasted body of [cargo_metadata::MetadataCommand::exec]
/// which was needed for more flexibility
pub fn exec_metadata_command(
    mut command: std::process::Command,
) -> cargo_metadata::Result<cargo_metadata::Metadata> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(cargo_metadata::Error::CargoMetadata {
            stderr: String::from_utf8(output.stderr)?,
        });
    }
    let stdout = std::str::from_utf8(&output.stdout)?
        .lines()
        .find(|line| line.starts_with('{'))
        .ok_or(cargo_metadata::Error::NoJson)?;
    cargo_metadata::MetadataCommand::parse(stdout)
}

/// Get the result of `cargo metadata`, together with the root package id.
// TODO #F: uncomment for `build_external_extended` method
#[allow(unused)]
fn get_cargo_metadata(
    manifest_path: &ManifestPath,
    no_locked: bool,
    cargo_target_dir: &common_buildtime_env::CargoTargetDir,
) -> eyre::Result<(cargo_metadata::Metadata, Package)> {
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Fetching cargo metadata for {}", manifest_path.path
    );
    let mut cmd = MetadataCommand::new();
    if !no_locked {
        cmd.other_options(["--locked".to_string()]);
    }

    let cmd = cmd.manifest_path(&manifest_path.path);
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Command execution:\n{}",
        pretty_print::indent_payload(&format!("{:#?}", cmd.cargo_command()))
    );
    let mut std_process_command = cmd.cargo_command();

    cargo_target_dir.into_std_command(&mut std_process_command);

    let metadata = exec_metadata_command(std_process_command);
    if let Err(cargo_metadata::Error::CargoMetadata { stderr }) = metadata.as_ref() {
        if stderr.contains("remove the --locked flag") {
            println!(
                "{}",
                "An error with Cargo.lock has been encountered...".yellow()
            );
            println!(
                "{}",
                "You can choose to disable `--locked` flag for downstream `cargo` command \
                by adding `--no-locked` flag OR by removing `--locked` flag"
                    .cyan()
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
        .wrap_err(
            "raw_metadata.root_package() returned None.\n\
            Command was likely called from a root of virtual workspace as current directory \
            and not from a contract's crate",
        )?
        .clone();
    Ok((metadata, root_package))
}
