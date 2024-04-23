use std::{
    process::{id, Command, ExitStatus},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::commands::build_command::INSIDE_DOCKER_ENV_KEY;
use crate::util;

use color_eyre::{eyre::ContextCompat, owo_colors::OwoColorize};

#[cfg(unix)]
use nix::unistd::{getgid, getuid};

use super::BuildContext;

mod cloned_repo;
mod git_checks;
mod metadata;

impl super::BuildCommand {
    pub(super) fn docker_run(
        self,
        context: BuildContext,
    ) -> color_eyre::eyre::Result<util::CompilationArtifact> {
        util::handle_step("Performing git checks...", || {
            match context {
                BuildContext::Deploy => {
                    let contract_path: camino::Utf8PathBuf = self.contract_path()?;
                    // TODO: clone to tmp folder and checkout specific revision must be separate steps
                    eprintln!(
                        "\n The URL of the remote repository:\n {}\n",
                        git_checks::remote_repo_url(&contract_path)?
                    );
                    Ok(())
                }
                BuildContext::Build => Ok(()),
            }
        })?;
        let cloned_repo = util::handle_step(
            "Cloning project repo to a temporary build site, removing uncommitted changes...",
            || cloned_repo::ClonedRepo::clone(&self),
        )?;

        let docker_build_meta =
            util::handle_step("Parsing and validating `Cargo.toml` metadata...", || {
                metadata::ReproducibleBuild::parse(cloned_repo.crate_metadata())
            })?;

        util::print_step("Running docker command step...");
        let out_dir_arg = self.out_dir.clone();
        let (status, docker_cmd) = self.docker_subprocess_step(docker_build_meta, &cloned_repo)?;

        if status.success() {
            util::print_success("Running docker command step (finished)");
            util::handle_step("Copying artifact from temporary build site...", || {
                cloned_repo.copy_artifact(out_dir_arg)
            })
        } else {
            println!();
            println!(
                "{}",
                format!(
                    "See output above ↑↑↑.\nSourceScan command `{:?}` failed with exit status: {status}.",
                    docker_cmd
                ).yellow()
            );
            println!(
                "{}",
                "You can choose to opt out into non-docker build behaviour by using `--no-docker` flag.".cyan()
            );

            Err(color_eyre::eyre::eyre!(
                "Reproducible build in docker container failed"
            ))
        }
    }

    fn docker_subprocess_step(
        self,
        docker_build_meta: metadata::ReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> color_eyre::eyre::Result<(ExitStatus, Command)> {
        let mut cargo_args = vec![];

        if self.no_abi {
            cargo_args.push("--no-abi");
        }
        if self.no_embed_abi {
            cargo_args.push("--no-embed-abi");
        }
        if self.no_doc {
            cargo_args.push("--no-doc");
        }
        if self.no_locked {
            return Err(color_eyre::eyre::eyre!("`--no-locked` flag is forbidden for reproducible builds in containers, because a specific Cargo.lock is required"));
        }
        if self.no_release {
            cargo_args.push("--no-release");
        }

        let color = self
            .color
            .clone()
            .unwrap_or(crate::common::ColorPreference::Auto)
            .to_string();
        cargo_args.extend(&["--color", &color]);
        // Cross-platform process ID and timestamp
        let pid = id().to_string();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let container_code_path = "/home/near/code".to_string();
        let volume = format!(
            "{}:{}",
            cloned_repo
                .tmp_repo
                .workdir()
                .wrap_err("Could not get the working directory for the repository")?
                .to_string_lossy(),
            &container_code_path
        );
        let docker_container_name = format!("cargo-near-{}-{}", timestamp, pid);
        let docker_image = docker_build_meta.concat_image();
        let near_build_env_ref = format!("{}={}", INSIDE_DOCKER_ENV_KEY, docker_image);

        // Platform-specific UID/GID retrieval
        #[cfg(unix)]
        let uid_gid = format!("{}:{}", getuid(), getgid());
        #[cfg(not(unix))]
        let uid_gid = "1000:1000".to_string();

        let mut docker_args = vec![
            "-u",
            &uid_gid,
            "-it",
            "--name",
            &docker_container_name,
            "--volume",
            &volume,
            "--rm",
            "--workdir",
            &container_code_path,
            "--env",
            &near_build_env_ref,
            "--env",
            "RUST_LOG=cargo_near=info",
            &docker_image,
            "/bin/bash",
            "-c",
        ];

        let mut cargo_cmd_list = vec!["cargo", "near", "build"];
        cargo_cmd_list.extend(&cargo_args);
        println!(
            " {} {}",
            "build command in container:".green(),
            cargo_cmd_list.join(" ")
        );

        let cargo_cmd = cargo_cmd_list.join(" ");

        docker_args.push(&cargo_cmd);
        log::debug!("docker command : {:?}", docker_args);

        let mut docker_cmd = Command::new("docker");
        docker_cmd.arg("run");
        docker_cmd.args(docker_args);

        let status = match docker_cmd.status() {
            Ok(exit_status) => exit_status,
            Err(io_err) => {
                println!();
                println!(
                    "{}",
                    format!(
                        "Error obtaining status from executing SourceScan command `{:?}`",
                        docker_cmd
                    )
                    .yellow()
                );
                println!("{}", format!("Error `{:?}`", io_err).yellow());
                return Err(color_eyre::eyre::eyre!(
                    "Reproducible build in docker container failed"
                ));
            }
        };
        Ok((status, docker_cmd))
    }
}