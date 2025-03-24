use super::BuildContext;
use std::marker::PhantomData;
use std::time::Duration;

use crate::pretty_print;
use crate::types::cargo::manifest_path::{ManifestPath, MANIFEST_FILE_NAME};
use crate::types::cargo::metadata::CrateMetadata;
use crate::types::near::build::output::version_info::VersionInfo;
use crate::types::near::build::side_effects::ArtifactMessages;
use crate::types::near::docker_build::WARN_BECOMES_ERR;
use crate::{camino, BuildArtifact};
use colored::Colorize;

use super::crate_in_repo;

const ERR_NO_LOCKED_DEPLOY: &str = "`--no-locked` flag is forbidden for deploy with docker.";

pub struct ClonedRepo {
    pub initial_crate_in_repo: crate_in_repo::Crate,
    #[allow(unused)]
    pub tmp_repo_dir: tempfile::TempDir,
    no_locked: bool,
    tmp_crate_metadata: CrateMetadata,
}

impl ClonedRepo {
    pub fn check_locked_then_clone(
        crate_in_repo: crate_in_repo::Crate,
        no_locked: bool,
        context: BuildContext,
    ) -> eyre::Result<Self> {
        match (no_locked, context) {
            (false, _) => {}
            (true, BuildContext::Build) => {
                no_locked_warn_pause(true);
                println!();
                println!("{}", WARN_BECOMES_ERR.red(),);
                std::thread::sleep(Duration::new(5, 0));
            }
            (true, BuildContext::Deploy { .. }) => {
                println!(
                    "{}",
                    "Check in Cargo.lock for contract being built into source control.".yellow()
                );
                return Err(eyre::eyre!(ERR_NO_LOCKED_DEPLOY));
            }
        }
        Self::git_clone(crate_in_repo, no_locked)
    }

    fn git_clone(crate_in_repo: crate_in_repo::Crate, no_locked: bool) -> eyre::Result<Self> {
        let tmp_repo_dir = tempfile::tempdir()?;
        let tmp_repo_path = tmp_repo_dir.path().to_path_buf();
        let tmp_repo =
            git2::Repository::clone_recurse(crate_in_repo.repo_root.as_str(), &tmp_repo_path)?;
        println!(
            "{} {:?}",
            format!("current HEAD ({}):", tmp_repo.path().display()).green(),
            tmp_repo.revparse_single("HEAD")?.id()
        );

        pretty_print::step("Collecting cargo project metadata from temporary build site...");
        let tmp_crate_metadata = {
            let cargo_toml_path: camino::Utf8PathBuf = {
                let mut path: camino::Utf8PathBuf = tmp_repo_path.clone().try_into()?;
                path.push(crate_in_repo.host_relative_path()?);
                path.push(MANIFEST_FILE_NAME);
                path
            };
            let manifest_path = ManifestPath::try_from(cargo_toml_path)?;
            CrateMetadata::collect(manifest_path, no_locked, None).inspect_err(|err| {
            if !no_locked && err.to_string().contains("Cargo.lock is absent") {
                no_locked_warn_pause(false);
                println!();
                println!("{}", "Cargo.lock check was performed against git version of code.".cyan());
                println!("{}", "Don't forget to check in Cargo.lock into source code for deploy if it's git-ignored...".cyan());
            }
        })?
        };
        tracing::info!(
            "obtained tmp_crate_metadata.target_directory: {}",
            tmp_crate_metadata.target_directory
        );

        Ok(ClonedRepo {
            tmp_repo_dir,
            no_locked,
            initial_crate_in_repo: crate_in_repo,
            tmp_crate_metadata,
        })
    }

    pub fn crate_metadata(&self) -> &CrateMetadata {
        &self.tmp_crate_metadata
    }
    pub fn contract_source_workdir(&self) -> eyre::Result<camino::Utf8PathBuf> {
        let path = camino::Utf8PathBuf::try_from(self.tmp_repo_dir.path().to_path_buf())?;
        Ok(path)
    }
    pub fn copy_artifact(
        self,
        cli_override: Option<camino::Utf8PathBuf>,
    ) -> eyre::Result<BuildArtifact> {
        let tmp_out_dir = self.tmp_crate_metadata.resolve_output_dir(None)?;

        let destination_crate_metadata = {
            let cargo_toml_path: camino::Utf8PathBuf = {
                let mut path = self.initial_crate_in_repo.crate_root.clone();
                path.push(MANIFEST_FILE_NAME);
                path
            };
            let manifest_path = ManifestPath::try_from(cargo_toml_path)?;
            CrateMetadata::collect(manifest_path, self.no_locked, None)?
        };

        let destination_dir = destination_crate_metadata.resolve_output_dir(cli_override)?;

        copy(tmp_out_dir, self.tmp_crate_metadata, destination_dir)
    }
}

fn copy(
    tmp_out_dir: camino::Utf8PathBuf,
    tmp_crate_metadata: CrateMetadata,
    mut destination_dir: camino::Utf8PathBuf,
) -> eyre::Result<BuildArtifact> {
    println!(
        " {} {}",
        "artifact search location in temporary build site:".green(),
        tmp_out_dir
    );

    let filename = format!("{}.wasm", tmp_crate_metadata.formatted_package_name());

    let in_wasm_path = tmp_out_dir.join(filename.clone());

    if !in_wasm_path.exists() {
        return Err(eyre::eyre!(
            "Temporary build site result wasm file not found: `{:?}`.",
            in_wasm_path
        ));
    }

    let out_wasm_path = {
        destination_dir.push(filename);
        destination_dir
    };
    if out_wasm_path.exists() {
        println!(" {}", "removing previous artifact".cyan());
        std::fs::remove_file(&out_wasm_path)?;
    }
    std::fs::copy::<camino::Utf8PathBuf, camino::Utf8PathBuf>(in_wasm_path, out_wasm_path.clone())?;
    let result = BuildArtifact {
        path: out_wasm_path,
        fresh: true,
        from_docker: true,
        builder_version_info: Some(VersionInfo::UnknownFromDocker),
        artifact_type: PhantomData,
    };
    let mut messages = ArtifactMessages::default();
    messages.push_binary(&result)?;
    messages.pretty_print();

    Ok(result)
}

fn no_locked_warn_pause(warning_red: bool) {
    println!();
    let warning = if warning_red {
        format!("{}", "WARNING: ".red())
    } else {
        "".to_string()
    };
    println!(
        "{}{}",
        warning,
        "Please mind that `--no-locked` flag is allowed in Docker builds, but:".cyan()
    );
    println!("{}", "  - such builds are not reproducible due to potential update of dependencies and compiled `wasm` mismatch as the result.".yellow());
    std::thread::sleep(Duration::new(12, 0));
}
