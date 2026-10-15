use super::BuildContext;
use std::marker::PhantomData;
use std::time::Duration;

use crate::pretty_print;
use crate::types::cargo::manifest_path::{MANIFEST_FILE_NAME, ManifestPath};
use crate::types::cargo::metadata::CrateMetadata;
use crate::types::near::build::common_buildtime_env::CargoTargetDir;
use crate::types::near::build::output::version_info::VersionInfo;
use crate::types::near::build::side_effects::ArtifactMessages;
use crate::types::near::docker_build::WARN_BECOMES_ERR;
use crate::{BuildArtifact, camino};
use colored::Colorize;
use eyre::WrapErr;

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
            CrateMetadata::collect(manifest_path, no_locked, &CargoTargetDir::NoOp, None).inspect_err(|err| {
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
        in_wasm_path: camino::Utf8PathBuf,
        cli_override: Option<camino::Utf8PathBuf>,
    ) -> eyre::Result<BuildArtifact> {
        let destination_crate_metadata = {
            let cargo_toml_path: camino::Utf8PathBuf = {
                let mut path = self.initial_crate_in_repo.crate_root.clone();
                path.push(MANIFEST_FILE_NAME);
                path
            };
            let manifest_path = ManifestPath::try_from(cargo_toml_path)?;
            CrateMetadata::collect(manifest_path, self.no_locked, &CargoTargetDir::NoOp, None)?
        };

        let destination_dir = destination_crate_metadata
            .get_legacy_cargo_near_output_path(cli_override)?
            .out_dir;

        let in_abi_path = in_wasm_path.with_file_name(format!(
            "{}_abi.json",
            self.tmp_crate_metadata.formatted_package_name()
        ));
        copy(in_wasm_path, in_abi_path, destination_dir)
    }
}

fn copy(
    in_wasm_path: camino::Utf8PathBuf,
    in_abi_path: camino::Utf8PathBuf,
    destination_dir: camino::Utf8PathBuf,
) -> eyre::Result<BuildArtifact> {
    let file_name = in_wasm_path
        .file_name()
        .expect("expected to be a wasm file path name as the result of [near_verify_rs::logic::nep330_build::run]");
    let out_wasm_path = destination_dir.join(file_name);
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
    // Custom builders and builds with ABI generation disabled may only produce WASM.
    match std::fs::metadata(&in_abi_path) {
        Ok(_) => {
            let out_abi_path = destination_dir.join(in_abi_path.file_name().unwrap());
            std::fs::copy(&in_abi_path, &out_abi_path).wrap_err_with(|| {
                format!("failed to copy ABI `{in_abi_path}` to `{out_abi_path}`")
            })?;
            messages.push_free(("ABI", out_abi_path.to_string().yellow().bold()));
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).wrap_err_with(|| format!("failed to read ABI `{in_abi_path}`"));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_abi_with_wasm() -> eyre::Result<()> {
        let source = tempfile::tempdir()?;
        let source = camino::Utf8Path::from_path(source.path()).unwrap();
        let wasm = source.join("test_contract.wasm");
        let abi = source.join("test_contract_abi.json");
        std::fs::write(&wasm, b"wasm bytes")?;
        let json = br#"{"metadata":{"wasm_hash":"final hash"}}"#;
        std::fs::write(&abi, json)?;
        let destination = tempfile::tempdir()?;
        let destination = camino::Utf8Path::from_path(destination.path()).unwrap();
        let destination = crate::fs::force_canonicalize_dir(&destination.join("nested/output"))?;
        std::fs::write(destination.join("test_contract.wasm"), b"old wasm")?;
        std::fs::write(destination.join("test_contract_abi.json"), b"old abi")?;

        let artifact = copy(wasm.clone(), abi, destination.clone())?;

        assert_eq!(artifact.path, destination.join("test_contract.wasm"));
        assert_eq!(std::fs::read(&artifact.path)?, std::fs::read(&wasm)?);
        assert_eq!(
            std::fs::read(destination.join("test_contract_abi.json"))?,
            json
        );
        assert!(artifact.fresh && artifact.from_docker);
        assert!(matches!(
            artifact.builder_version_info,
            Some(VersionInfo::UnknownFromDocker)
        ));
        Ok(())
    }

    #[test]
    fn exports_custom_wasm_without_abi() -> eyre::Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = camino::Utf8Path::from_path(dir.path()).unwrap();
        let wasm = dir.join("custom.wasm");
        std::fs::write(&wasm, b"custom wasm")?;
        let destination = crate::fs::force_canonicalize_dir(&dir.join("output"))?;

        let artifact = copy(
            wasm,
            dir.join("test_contract_abi.json"),
            destination.clone(),
        )?;

        assert_eq!(artifact.path, destination.join("custom.wasm"));
        assert_eq!(std::fs::read(artifact.path)?, b"custom wasm");
        assert!(!destination.join("test_contract_abi.json").exists());
        Ok(())
    }

    #[test]
    fn propagates_abi_io_errors() -> eyre::Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = camino::Utf8Path::from_path(dir.path()).unwrap();
        let wasm = dir.join("test_contract.wasm");
        let abi = dir.join("test_contract_abi.json");
        std::fs::write(&wasm, b"wasm bytes")?;
        let destination = crate::fs::force_canonicalize_dir(&dir.join("output"))?;
        std::fs::create_dir(&abi)?;
        assert!(copy(wasm.clone(), abi.clone(), destination.clone()).is_err());
        std::fs::remove_dir(&abi)?;
        std::fs::write(&abi, b"abi bytes")?;
        std::fs::create_dir(destination.join("test_contract_abi.json"))?;
        assert!(copy(wasm.clone(), abi, destination.clone()).is_err());

        // An invalid source parent must not be treated as an absent ABI.
        assert!(copy(wasm.clone(), wasm.join("abi.json"), destination).is_err());
        Ok(())
    }

    #[test]
    fn missing_wasm_still_fails() -> eyre::Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = camino::Utf8Path::from_path(dir.path()).unwrap();
        assert!(
            copy(
                dir.join("missing.wasm"),
                dir.join("missing_abi.json"),
                dir.into()
            )
            .is_err()
        );
        Ok(())
    }
}
