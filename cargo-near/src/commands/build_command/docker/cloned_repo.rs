use crate::{
    commands::build_command::ArtifactMessages,
    types::{
        manifest::{CargoManifestPath, MANIFEST_FILE_NAME},
        metadata::CrateMetadata,
    },
    util::{self, CompilationArtifact},
};
use camino::Utf8PathBuf;
use colored::Colorize;

use super::crate_in_repo;

pub(super) struct ClonedRepo {
    pub initial_crate_in_repo: crate_in_repo::Crate,
    #[allow(unused)]
    pub tmp_repo_dir: tempfile::TempDir,
    no_locked: bool,
    tmp_crate_metadata: CrateMetadata,
}

impl ClonedRepo {
    pub(super) fn git_clone(
        crate_in_repo: crate_in_repo::Crate,
        no_locked: bool,
    ) -> color_eyre::eyre::Result<Self> {
        let tmp_repo_dir = tempfile::tempdir()?;
        let tmp_repo_path = tmp_repo_dir.path().to_path_buf();
        let tmp_repo =
            git2::Repository::clone_recurse(crate_in_repo.repo_root.as_str(), &tmp_repo_path)?;
        println!(
            "{} {:?}",
            format!("current HEAD ({}):", tmp_repo.path().display()).green(),
            tmp_repo.revparse_single("HEAD")?.id()
        );

        util::print_step("Collecting cargo project metadata from temporary build site...");
        let tmp_crate_metadata = {
            let cargo_toml_path: camino::Utf8PathBuf = {
                let mut path: camino::Utf8PathBuf = tmp_repo_path.clone().try_into()?;
                path.push(crate_in_repo.host_relative_path()?);
                path.push(MANIFEST_FILE_NAME);
                path
            };
            CrateMetadata::collect(CargoManifestPath::try_from(cargo_toml_path)?, no_locked).map_err(|err| {
            if !no_locked && err.to_string().contains("Cargo.lock is absent") {
                super::no_locked_warn_pause(false);
                println!();
                println!("{}", "Cargo.lock check was performed against git version of code.".cyan());
                println!("{}", "Don't forget to check in Cargo.lock into source code for deploy if it's git-ignored...".cyan());
            }
            err
        })?
        };
        log::info!(
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

    pub(super) fn crate_metadata(&self) -> &CrateMetadata {
        &self.tmp_crate_metadata
    }
    pub(super) fn copy_artifact(
        self,
        cli_override: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    ) -> color_eyre::eyre::Result<CompilationArtifact> {
        let tmp_out_dir = self.tmp_crate_metadata.resolve_output_dir(None)?;

        let destination_crate_metadata = {
            let cargo_toml_path: camino::Utf8PathBuf = {
                let mut path = self.initial_crate_in_repo.crate_root.clone();
                path.push(MANIFEST_FILE_NAME);
                path
            };
            CrateMetadata::collect(
                CargoManifestPath::try_from(cargo_toml_path)?,
                self.no_locked,
            )?
        };

        let destination_dir = destination_crate_metadata.resolve_output_dir(cli_override)?;

        copy(tmp_out_dir, self.tmp_crate_metadata, destination_dir)
    }
}

fn copy(
    tmp_out_dir: Utf8PathBuf,
    tmp_crate_metadata: CrateMetadata,
    mut destination_dir: Utf8PathBuf,
) -> color_eyre::eyre::Result<CompilationArtifact> {
    println!(
        " {} {}",
        "artifact search location in temporary build site:".green(),
        tmp_out_dir
    );

    let filename = format!("{}.wasm", tmp_crate_metadata.formatted_package_name());

    let in_wasm_path = tmp_out_dir.join(filename.clone());

    if !in_wasm_path.exists() {
        return Err(color_eyre::eyre::eyre!(
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
    let result = CompilationArtifact {
        path: out_wasm_path,
        fresh: true,
        from_docker: true,
        cargo_near_version_mismatch: None,
    };
    let mut messages = ArtifactMessages::default();
    messages.push_binary(&result)?;
    messages.pretty_print();

    Ok(result)
}
