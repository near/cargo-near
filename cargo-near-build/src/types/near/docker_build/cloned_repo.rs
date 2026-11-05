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

use super::crate_in_repo;

const ERR_NO_LOCKED_DEPLOY: &str = "`--no-locked` flag is forbidden for deploy with docker.";

pub struct ClonedRepo {
    pub initial_crate_in_repo: crate_in_repo::Crate,
    #[allow(unused)]
    pub tmp_repo_dir: tempfile::TempDir,
    no_locked: bool,
    offline_build: bool,
    tmp_crate_metadata: CrateMetadata,
}

impl ClonedRepo {
    pub fn check_locked_then_clone(
        crate_in_repo: crate_in_repo::Crate,
        no_locked: bool,
        context: BuildContext,
        offline_build: bool,
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
        Self::git_clone(crate_in_repo, no_locked, offline_build)
    }

    fn git_clone(
        crate_in_repo: crate_in_repo::Crate,
        no_locked: bool,
        offline_build: bool,
    ) -> eyre::Result<Self> {
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
            collect_crate_metadata(manifest_path, no_locked, offline_build).inspect_err(|err| {
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
            offline_build,
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
            collect_crate_metadata(manifest_path, self.no_locked, self.offline_build)?
        };

        let destination_dir = destination_crate_metadata
            .get_legacy_cargo_near_output_path(cli_override)?
            .out_dir;

        copy(in_wasm_path, destination_dir)
    }
}

fn collect_crate_metadata(
    manifest_path: ManifestPath,
    no_locked: bool,
    offline_build: bool,
) -> eyre::Result<CrateMetadata> {
    if offline_build {
        CrateMetadata::collect_for_offline_docker(manifest_path, no_locked)
    } else {
        CrateMetadata::collect(manifest_path, no_locked, &CargoTargetDir::NoOp, None)
    }
}

fn copy(
    in_wasm_path: camino::Utf8PathBuf,
    mut destination_dir: camino::Utf8PathBuf,
) -> eyre::Result<BuildArtifact> {
    let file_name = in_wasm_path
        .file_name()
        .expect("expected to be a wasm file path name as the result of [near_verify_rs::logic::nep330_build::run]");
    let out_wasm_path = {
        destination_dir.push(file_name);
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt, process::Command};

    #[test]
    fn offline_metadata_does_not_execute_repository_wrapper() -> eyre::Result<()> {
        const CHILD: &str = "CARGO_NEAR_OFFLINE_METADATA_TEST_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let root = camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?).unwrap();
            let manifest = ManifestPath::try_from(root.join("Cargo.toml"))?;
            let marker = root.join("wrapper-ran");

            // Positive control: normal metadata executes the repository wrapper
            // for compiler probes. The fixture wrapper records this and exits.
            assert!(collect_crate_metadata(manifest.clone(), true, false).is_err());
            assert!(
                marker.exists(),
                "online control did not execute the wrapper"
            );
            fs::remove_file(&marker)?;

            let metadata = collect_crate_metadata(manifest, false, true)?;
            assert_eq!(metadata.root_package.name.as_str(), "offline-metadata");
            assert!(metadata.raw_metadata.resolve.is_none());
            assert!(
                !marker.exists(),
                "offline host metadata executed the wrapper"
            );
            assert!(!root.join("Cargo.lock").exists());
            return Ok(());
        }

        let directory = tempfile::tempdir()?;
        let root = directory.path();
        fs::create_dir(root.join("src"))?;
        fs::create_dir(root.join(".cargo"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"offline-metadata\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(root.join("src/lib.rs"), "pub fn answer() -> u32 { 42 }\n")?;
        let wrapper = root.join("wrapper.sh");
        fs::write(
            &wrapper,
            "#!/bin/sh\ntouch \"$(dirname \"$0\")/wrapper-ran\"\nexit 97\n",
        )?;
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755))?;
        fs::write(
            root.join(".cargo/config.toml"),
            format!("[build]\nrustc-wrapper = {:?}\n", wrapper),
        )?;

        // Use a child test process so Cargo really starts inside the untrusted
        // repository, without mutating the parent's CWD or environment.
        let output = Command::new(std::env::current_exe()?)
            .arg("offline_metadata_does_not_execute_repository_wrapper")
            .arg("--nocapture")
            .current_dir(root)
            .env(CHILD, "1")
            .env_remove("RUSTC_WRAPPER")
            .env_remove("RUSTC_WORKSPACE_WRAPPER")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && stdout.contains("1 passed"),
            "child metadata test failed:\n{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
