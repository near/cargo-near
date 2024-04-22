use std::ffi::OsStr;

use crate::{
    commands::build_command::{ArtifactMessages, BuildCommand},
    types::{
        manifest::{CargoManifestPath, MANIFEST_FILE_NAME},
        metadata::CrateMetadata,
    },
    util::{self, CompilationArtifact},
};
use camino::Utf8PathBuf;
use color_eyre::{eyre::WrapErr, owo_colors::OwoColorize};

pub(super) struct ClonedRepo {
    pub tmp_repo: git2::Repository,
    pub contract_path: camino::Utf8PathBuf,
    #[allow(unused)]
    tmp_contract_dir: tempfile::TempDir,
    tmp_crate_metadata: CrateMetadata,
}

impl ClonedRepo {
    pub(super) fn clone(args: &BuildCommand) -> color_eyre::eyre::Result<Self> {
        let contract_path: camino::Utf8PathBuf = args.contract_path()?;
        log::info!("ClonedRepo.contract_path: {:?}", contract_path,);

        let tmp_contract_dir = tempfile::tempdir()?;
        let tmp_contract_path = tmp_contract_dir.path().to_path_buf();
        log::info!("ClonedRepo.tmp_contract_path: {:?}", tmp_contract_path);
        let tmp_repo = git2::Repository::clone(contract_path.as_str(), &tmp_contract_path)?;

        util::print_step("Collecting cargo project metadata from temporary build site...");
        let tmp_crate_metadata = {
            let cargo_toml_path: camino::Utf8PathBuf = {
                let mut cloned_path: std::path::PathBuf = tmp_contract_path.clone();
                cloned_path.push(MANIFEST_FILE_NAME);
                cloned_path.try_into()?
            };
            CrateMetadata::collect(CargoManifestPath::try_from(cargo_toml_path)?, false)?
        };

        Ok(ClonedRepo {
            tmp_repo,
            tmp_contract_dir,
            contract_path,
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
                let mut cloned_path = self.contract_path.clone();
                cloned_path.push(MANIFEST_FILE_NAME);
                cloned_path
            };
            CrateMetadata::collect(CargoManifestPath::try_from(cargo_toml_path)?, false)?
        };

        let destination_dir = destination_crate_metadata.resolve_output_dir(cli_override)?;

        search_and_copy(tmp_out_dir, destination_dir)
    }
}

fn search_and_copy(
    tmp_out_dir: Utf8PathBuf,
    mut destination_dir: Utf8PathBuf,
) -> color_eyre::eyre::Result<CompilationArtifact> {
    println!(
        " {} {}",
        "artifact search location in temporary build site:".green(),
        tmp_out_dir
    );

    let dir = tmp_out_dir
        .read_dir()
        .wrap_err_with(|| format!("No artifacts directory found: `{:?}`.", tmp_out_dir))?;

    for entry in dir.flatten() {
        if entry
            .path()
            .extension()
            .unwrap_or(OsStr::new("not_wasm"))
            .to_str()
            .unwrap_or("not_wasm")
            == "wasm"
        {
            let out_wasm_path = {
                let filename = entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                destination_dir.push(filename);
                destination_dir
            };
            if out_wasm_path.exists() {
                println!(" {}", "removing previous artifact".cyan());
                std::fs::remove_file(&out_wasm_path)?;
            }
            std::fs::copy::<std::path::PathBuf, camino::Utf8PathBuf>(
                entry.path(),
                out_wasm_path.clone(),
            )?;
            let result = CompilationArtifact {
                path: out_wasm_path,
                fresh: true,
                from_docker: true,
            };
            let mut messages = ArtifactMessages::default();
            messages.push_binary(&result);
            messages.pretty_print();

            return Ok(result);
        }
    }

    Err(color_eyre::eyre::eyre!(
        "Wasm file not found in directory: `{:?}`.",
        tmp_out_dir
    ))
}
