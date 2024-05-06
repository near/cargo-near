use std::{
    process::{id, Command, ExitStatus},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{commands::build_command::CONTRACT_PATH_ENV_KEY, types::source_id, util};
use crate::{commands::build_command::INSIDE_DOCKER_ENV_KEY, common::ColorPreference};

use color_eyre::eyre::ContextCompat;

use colored::Colorize;
#[cfg(unix)]
use nix::unistd::{getgid, getuid};

use super::{BuildContext, REPO_LINK_HINT_ENV_KEY, SOURCE_CODE_SNAPSHOT_ENV_KEY};

mod cloned_repo;
mod crate_in_repo;
mod git_checks;
mod metadata;

impl super::BuildCommand {
    pub(super) fn docker_run(
        self,
        context: BuildContext,
    ) -> color_eyre::eyre::Result<util::CompilationArtifact> {
        let color = self.color.clone().unwrap_or(ColorPreference::Auto);
        color.apply();
        let crate_in_repo = util::handle_step(
            "Opening repo and determining HEAD and relative path of contract...",
            || crate_in_repo::Crate::find(&self.contract_path()?),
        )?;
        util::handle_step("Checking if git is dirty...", || {
            Self::git_dirty_check(context, &crate_in_repo.repo_root)
        })?;
        let cloned_repo = util::handle_step(
            "Cloning project repo to a temporary build site, removing uncommitted changes...",
            || cloned_repo::ClonedRepo::git_clone(crate_in_repo.clone()),
        )?;

        let docker_build_meta =
            util::handle_step("Parsing and validating `Cargo.toml` metadata...", || {
                metadata::ReproducibleBuild::parse(cloned_repo.crate_metadata())
            })?;

        if let BuildContext::Deploy = context {
            util::handle_step(
                "Performing check that current HEAD has been pushed to remote...",
                || {
                    git_checks::pushed_to_remote::check(
                        &docker_build_meta.source_code_git_url,
                        crate_in_repo.head,
                    )
                },
            )?;
        }

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
        let mut docker_cmd: Command = {
            // Platform-specific UID/GID retrieval
            #[cfg(unix)]
            let uid_gid = format!("{}:{}", getuid(), getgid());
            #[cfg(not(unix))]
            let uid_gid = "1000:1000".to_string();

            let docker_container_name = {
                // Cross-platform process ID and timestamp
                let pid = id().to_string();
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string();
                format!("cargo-near-{}-{}", timestamp, pid)
            };
            let container_paths = ContainerPaths::compute(cloned_repo)?;
            let docker_image = docker_build_meta.concat_image();
            println!(" {} {}", "docker image to be used:".green(), docker_image);

            let env = EnvVars::new(&docker_build_meta, cloned_repo)?;
            let env_args = env.docker_args();
            let cargo_cmd =
                self.compute_build_command(docker_build_meta.container_build_command.clone())?;
            println!(" {} {}", "build command in container:".green(), cargo_cmd);

            let docker_args = {
                let mut docker_args = vec![
                    "-u",
                    &uid_gid,
                    "-it",
                    "--name",
                    &docker_container_name,
                    "--volume",
                    &container_paths.host_volume_arg,
                    "--rm",
                    "--workdir",
                    &container_paths.crate_path,
                ];

                docker_args.extend(env_args.iter().map(|string| string.as_str()));

                docker_args.extend(vec![&docker_image, "/bin/bash", "-c"]);

                docker_args.push(&cargo_cmd);
                log::debug!("docker command : {:?}", docker_args);
                docker_args
            };

            let mut docker_cmd = Command::new("docker");
            docker_cmd.arg("run");
            docker_cmd.args(docker_args);
            docker_cmd
        };

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

    const BUILD_COMMAND_CLI_CONFIG_ERR: &'static str =  "cannot be used, when `container_build_command` is configured from `[package.metadata.near.reproducible_build]` in Cargo.toml";

    fn compute_build_command(
        &self,
        manifest_command: Option<String>,
    ) -> color_eyre::eyre::Result<String> {
        if let Some(cargo_cmd) = manifest_command {
            if self.no_abi {
                return Err(color_eyre::eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-abi",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_embed_abi {
                return Err(color_eyre::eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-embed-abi",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_doc {
                return Err(color_eyre::eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-doc",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_locked {
                return Err(color_eyre::eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-locked",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_release {
                return Err(color_eyre::eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-release",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            return Ok(cargo_cmd);
        }
        println!(
            " {}",
            "configuring `container_build_command` from cli args, passed to current command".cyan()
        );
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

        let color;
        if let Some(ref color_arg) = self.color {
            color = color_arg.to_string();
            cargo_args.extend(&["--color", &color]);
        }
        let mut cargo_cmd_list = vec!["cargo", "near", "build"];
        cargo_cmd_list.extend(&cargo_args);
        let cargo_cmd = cargo_cmd_list.join(" ");
        Ok(cargo_cmd)
    }

    fn git_dirty_check(
        context: BuildContext,
        repo_root: &camino::Utf8PathBuf,
    ) -> color_eyre::eyre::Result<()> {
        let result = git_checks::dirty::check(repo_root);
        match (result, context) {
            (Err(err), BuildContext::Deploy) => {
                println!(
                    " {}",
                    "Either commit and push, or revert following changes to continue deployment:"
                        .yellow()
                );
                Err(err)
            }
            (Err(err), BuildContext::Build) => {
                println!(
                    "{}",
                    util::indent_string(&format!("{}: {}", "WARNING".yellow(), err))
                );
                println!(
                    " {}",
                    "This WARNING becomes a hard ERROR when deploying!".yellow(),
                );

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

struct ContainerPaths {
    host_volume_arg: String,
    crate_path: String,
}

const NEP330_REPO_MOUNT: &str = "/home/near/code";

impl ContainerPaths {
    fn compute(cloned_repo: &cloned_repo::ClonedRepo) -> color_eyre::eyre::Result<Self> {
        let mounted_repo = NEP330_REPO_MOUNT.to_string();
        let host_volume_arg = format!(
            "{}:{}",
            cloned_repo.tmp_repo_dir.path().to_string_lossy(),
            &mounted_repo
        );
        let crate_path = {
            let mut repo_path = unix_path::Path::new(NEP330_REPO_MOUNT).to_path_buf();
            repo_path.push(cloned_repo.initial_crate_in_repo.unix_relative_path()?);

            repo_path
                .to_str()
                .wrap_err("non UTF-8 unix path computed as crate path")?
                .to_string()
        };
        Ok(Self {
            host_volume_arg,
            crate_path,
        })
    }
}

const RUST_LOG_EXPORT: &str = "RUST_LOG=cargo_near=info";

struct Nep330BuildInfo {
    build_environment: String,
    contract_path: Option<String>,
    source_code_snapshot: source_id::SourceId,
}

impl Nep330BuildInfo {
    fn new(
        docker_build_meta: &metadata::ReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> color_eyre::eyre::Result<Self> {
        let build_environment = docker_build_meta.concat_image();
        let contract_path = cloned_repo
            .initial_crate_in_repo
            .unix_relative_path()?
            .to_str()
            .wrap_err("non UTF-8 unix path computed as contract path")?
            .to_string();
        let contract_path = if contract_path.is_empty() {
            None
        } else {
            Some(contract_path)
        };

        let source_code_snapshot = source_id::SourceId::for_git(
            &docker_build_meta.source_code_git_url,
            source_id::GitReference::Rev(cloned_repo.initial_crate_in_repo.head.to_string()),
        )
        .map_err(|err| color_eyre::eyre::eyre!("compute SourceId {}", err))?;
        Ok(Self {
            build_environment,
            contract_path,
            source_code_snapshot,
        })
    }

    fn docker_args(&self) -> Vec<String> {
        let mut result = vec![
            "--env".to_string(),
            format!("{}={}", INSIDE_DOCKER_ENV_KEY, self.build_environment),
            "--env".to_string(),
            format!(
                "{}={}",
                SOURCE_CODE_SNAPSHOT_ENV_KEY,
                self.source_code_snapshot.as_url()
            ),
        ];

        if let Some(ref contract_path) = self.contract_path {
            result.extend(vec![
                "--env".to_string(),
                format!("{}={}", CONTRACT_PATH_ENV_KEY, contract_path),
            ]);
        }

        result
    }
}
struct EnvVars {
    build_info: Nep330BuildInfo,
    rust_log: String,
    repo_link: Option<String>,
    revision: String,
}

impl EnvVars {
    fn new(
        docker_build_meta: &metadata::ReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> color_eyre::eyre::Result<Self> {
        let build_info = Nep330BuildInfo::new(docker_build_meta, cloned_repo)?;
        let repo_link = cloned_repo.crate_metadata().root_package.repository.clone();
        let revision = cloned_repo.initial_crate_in_repo.head.to_string();
        Ok(Self {
            build_info,
            rust_log: RUST_LOG_EXPORT.to_string(),
            repo_link,
            revision,
        })
    }

    fn docker_args(&self) -> Vec<String> {
        let mut result = self.build_info.docker_args();

        if let Some(repo_link_hint) = self.compute_repo_link_hint() {
            result.extend(vec![
                "--env".to_string(),
                format!("{}={}", REPO_LINK_HINT_ENV_KEY, repo_link_hint,),
            ]);
        }
        result.extend(vec!["--env".to_string(), self.rust_log.clone()]);
        result
    }
    fn compute_repo_link_hint(&self) -> Option<String> {
        let url = self
            .repo_link
            .clone()
            .and_then(|repo_link| url::Url::parse(&repo_link).ok());

        let url = url
            .clone()
            .and_then(|url| url.host_str().map(ToString::to_string))
            .filter(|host| *host == "github.com")
            .and(url);
        let commit_hint = url.and_then(|url| {
            let existing_path = url.path();
            url.join(&format!("{}/tree/{}", existing_path, self.revision))
                .ok()
        });
        commit_hint.map(|url| url.to_string())
    }
}
