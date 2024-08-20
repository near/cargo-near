use crate::types::source_id;
use cargo_near_build::{camino, ManifestPath};
use cargo_near_build::{env_keys, pretty_print, BuildArtifact};
use std::ops::Deref;
use std::{
    io::IsTerminal,
    process::{id, Command, ExitStatus},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use color_eyre::eyre::ContextCompat;

use colored::Colorize;
#[cfg(target_os = "linux")]
use nix::unistd::{getgid, getuid};

use super::BuildContext;

mod cloned_repo;
mod crate_in_repo;
mod docker_checks;
mod git_checks;
mod metadata;

const ERR_REPRODUCIBLE: &str = "Reproducible build in docker container failed.";
const ERR_NO_LOCKED_DEPLOY: &str = "`--no-locked` flag is forbidden for deploy with docker.";
const WARN_BECOMES_ERR: &str =
    "This WARNING becomes a hard ERROR when deploying contract with docker.";

pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    pub no_locked: bool,
    /// Build contract in debug mode, without optimizations and bigger is size
    pub no_release: bool,
    /// Do not generate ABI for the contract
    pub no_abi: bool,
    /// Do not embed the ABI in the contract binary
    pub no_embed_abi: bool,
    /// Do not include rustdocs in the embedded ABI
    pub no_doc: bool,
    /// Copy final artifacts to this directory
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Set compile-time feature flags.
    pub features: Option<String>,
    /// Disables default feature flags.
    pub no_default_features: bool,
    /// Coloring: auto, always, never
    pub color: Option<cargo_near_build::ColorPreference>,
}

impl From<super::BuildCommand> for Opts {
    fn from(value: super::BuildCommand) -> Self {
        Self {
            no_locked: value.no_locked,
            no_release: value.no_release,
            no_abi: value.no_abi,
            no_embed_abi: value.no_embed_abi,
            no_doc: value.no_doc,
            features: value.features,
            no_default_features: value.no_default_features,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            color: value.color.map(Into::into),
        }
    }
}

impl Opts {
    pub fn contract_path(&self) -> color_eyre::eyre::Result<camino::Utf8PathBuf> {
        let contract_path: camino::Utf8PathBuf = if let Some(manifest_path) = &self.manifest_path {
            let manifest_path = ManifestPath::try_from(manifest_path.deref().clone())?;
            manifest_path.directory()?.to_path_buf()
        } else {
            camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?).map_err(|err| {
                color_eyre::eyre::eyre!("Failed to convert path {}", err.to_string_lossy())
            })?
        };
        Ok(contract_path)
    }

    pub(super) fn docker_run(
        self,
        context: BuildContext,
    ) -> color_eyre::eyre::Result<BuildArtifact> {
        let color = self
            .color
            .clone()
            .unwrap_or(cargo_near_build::ColorPreference::Auto);
        color.apply();
        let crate_in_repo = pretty_print::handle_step(
            "Opening repo and determining HEAD and relative path of contract...",
            || crate_in_repo::Crate::find(&self.contract_path()?),
        )?;
        pretty_print::handle_step("Checking if git is dirty...", || {
            Self::git_dirty_check(context, &crate_in_repo.repo_root)
        })?;
        let cloned_repo = pretty_print::handle_step(
            "Cloning project repo to a temporary build site, removing uncommitted changes...",
            || {
                match (self.no_locked, context) {
                    (false, _) => {}
                    (true, BuildContext::Build) => {
                        no_locked_warn_pause(true);
                        println!();
                        println!("{}", WARN_BECOMES_ERR.red(),);
                        thread::sleep(Duration::new(5, 0));
                    }
                    (true, BuildContext::Deploy) => {
                        println!(
                            "{}",
                            "Check in Cargo.lock for contract being built into source control."
                                .yellow()
                        );
                        return Err(color_eyre::eyre::eyre!(ERR_NO_LOCKED_DEPLOY));
                    }
                }
                cloned_repo::ClonedRepo::git_clone(crate_in_repo.clone(), self.no_locked)
            },
        )?;

        let docker_build_meta =
            pretty_print::handle_step("Parsing and validating `Cargo.toml` metadata...", || {
                metadata::ReproducibleBuild::parse(cloned_repo.crate_metadata())
            })?;

        if let BuildContext::Deploy = context {
            pretty_print::handle_step(
                "Performing check that current HEAD has been pushed to remote...",
                || {
                    git_checks::pushed_to_remote::check(
                        // this unwrap depends on `metadata::ReproducibleBuild::validate` logic
                        &docker_build_meta.repository.clone().unwrap(),
                        crate_in_repo.head,
                    )
                },
            )?;
        }
        if std::env::var(env_keys::nep330::nonspec::SERVER_DISABLE_INTERACTIVE).is_err() {
            pretty_print::handle_step("Performing `docker` sanity check...", || {
                docker_checks::sanity_check()
            })?;

            pretty_print::handle_step("Checking that specified image is available...", || {
                docker_checks::pull_image(&docker_build_meta)
            })?;
        }

        pretty_print::step("Running build in docker command step...");
        let out_dir_arg = self.out_dir.clone();
        let (status, docker_cmd) =
            self.docker_run_subprocess_step(docker_build_meta, &cloned_repo)?;

        Self::handle_docker_run_status(status, docker_cmd, cloned_repo, out_dir_arg)
    }

    fn docker_run_subprocess_step(
        self,
        docker_build_meta: metadata::ReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> color_eyre::eyre::Result<(ExitStatus, Command)> {
        let mut docker_cmd: Command = {
            // Platform-specific UID/GID retrieval

            // reason for this mapping is that on Linux the volume is mounted natively,
            // and thus the unprivileged user inside Docker container should be able to write
            // to the mounted folder that has the host user permissions,
            // not specifying this mapping results in UID=Docker-User owned files created in host system
            #[cfg(target_os = "linux")]
            let uid_gid = format!("{}:{}", getuid(), getgid());
            #[cfg(not(target_os = "linux"))]
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

            let env = EnvVars::new(&docker_build_meta, cloned_repo)?;
            let env_args = env.docker_args();
            let cargo_cmd = self.get_cli_build_command_in_docker(
                docker_build_meta.container_build_command.clone(),
            )?;
            println!("{} {}", "build command in container:".green(), cargo_cmd);

            let docker_args = {
                let mut docker_args = vec![
                    "-u",
                    &uid_gid,
                    "--name",
                    &docker_container_name,
                    "--volume",
                    &container_paths.host_volume_arg,
                    "--rm",
                    "--workdir",
                    &container_paths.crate_path,
                ];
                let stdin_is_terminal = std::io::stdin().is_terminal();
                log::debug!("input device is a tty: {}", stdin_is_terminal);
                if stdin_is_terminal
                    && std::env::var(env_keys::nep330::nonspec::SERVER_DISABLE_INTERACTIVE).is_err()
                {
                    docker_args.push("-it");
                }

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
        let status_result = docker_cmd.status();
        let status = docker_checks::handle_command_io_error(
            &docker_cmd,
            status_result,
            color_eyre::eyre::eyre!(ERR_REPRODUCIBLE),
        )?;

        Ok((status, docker_cmd))
    }

    fn handle_docker_run_status(
        status: ExitStatus,
        command: Command,
        cloned_repo: cloned_repo::ClonedRepo,
        out_dir_arg: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    ) -> color_eyre::eyre::Result<BuildArtifact> {
        if status.success() {
            pretty_print::success("Running docker command step (finished)");
            pretty_print::handle_step("Copying artifact from temporary build site...", || {
                cloned_repo.copy_artifact(out_dir_arg)
            })
        } else {
            docker_checks::print_command_status(status, command);
            Err(color_eyre::eyre::eyre!(ERR_REPRODUCIBLE))
        }
    }

    const BUILD_COMMAND_CLI_CONFIG_ERR: &'static str =  "cannot be used, when `container_build_command` is configured from `[package.metadata.near.reproducible_build]` in Cargo.toml";

    fn get_cli_build_command_in_docker(
        &self,
        manifest_command: Option<Vec<String>>,
    ) -> color_eyre::eyre::Result<String> {
        if let Some(cargo_cmd) = manifest_command {
            // NOTE: `--no-locked` is allowed for docker builds
            // if self.no_locked {
            //     no-op
            // }
            if self.no_release {
                return Err(color_eyre::eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-release",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
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
            if self.features.is_some() {
                return Err(color_eyre::eyre::eyre!(format!(
                    "`{}` {}",
                    "--features",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_default_features {
                return Err(color_eyre::eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-default-features",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            return Ok(cargo_cmd.join(" "));
        }
        println!(
            "{}",
            "configuring `container_build_command` from cli args, passed to current command".cyan()
        );
        let mut cargo_args = vec![];
        // NOTE: not passing through `no_locked` to cmd in container,
        // an invisible Cargo.lock was generated by implicit `cargo metadata` anyway
        // if self.no_locked {
        //     no-op
        // }
        if self.no_release {
            cargo_args.push("--no-release");
        }
        if self.no_abi {
            cargo_args.push("--no-abi");
        }
        if self.no_embed_abi {
            cargo_args.push("--no-embed-abi");
        }
        if self.no_doc {
            cargo_args.push("--no-doc");
        }
        if let Some(ref features) = self.features {
            cargo_args.extend(&["--features", features]);
        }
        if self.no_default_features {
            cargo_args.push("--no-default-features");
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
                    "{}",
                    "Either commit and push, or revert following changes to continue deployment:"
                        .yellow()
                );
                Err(err)
            }
            (Err(err), BuildContext::Build) => {
                println!();
                println!("{}: {}", "WARNING".red(), format!("{}", err).yellow());
                thread::sleep(Duration::new(3, 0));
                println!();
                println!("{}", WARN_BECOMES_ERR.red(),);
                // this is magic to help user notice:
                thread::sleep(Duration::new(5, 0));

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
    contract_path: String,
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

        let source_code_snapshot = source_id::SourceId::for_git(
            // this unwrap depends on `metadata::ReproducibleBuild::validate` logic
            docker_build_meta.repository.as_ref().unwrap(),
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
            format!(
                "{}={}",
                env_keys::nep330::BUILD_ENVIRONMENT,
                self.build_environment
            ),
            "--env".to_string(),
            format!(
                "{}={}",
                env_keys::nep330::SOURCE_CODE_SNAPSHOT,
                self.source_code_snapshot.as_url()
            ),
        ];

        result.extend(vec![
            "--env".to_string(),
            format!("{}={}", env_keys::nep330::CONTRACT_PATH, self.contract_path),
        ]);

        result
    }
}
struct EnvVars {
    build_info: Nep330BuildInfo,
    rust_log: String,
    repo_link: url::Url,
    revision: String,
}

impl EnvVars {
    fn new(
        docker_build_meta: &metadata::ReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> color_eyre::eyre::Result<Self> {
        let build_info = Nep330BuildInfo::new(docker_build_meta, cloned_repo)?;
        // this unwrap depends on `metadata::ReproducibleBuild::validate` logic
        let repo_link = docker_build_meta.repository.clone().unwrap();
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
                format!("{}={}", env_keys::nep330::LINK, repo_link_hint,),
            ]);
        }
        result.extend(vec!["--env".to_string(), self.rust_log.clone()]);
        result
    }
    fn compute_repo_link_hint(&self) -> Option<String> {
        let url = self.repo_link.clone();

        if url.host_str() == Some("github.com") {
            let existing_path = url.path();
            let existing_path = if existing_path.ends_with(".git") {
                existing_path.trim_end_matches(".git")
            } else {
                existing_path
            };

            Some(
                url.join(&format!("{}/tree/{}", existing_path, self.revision))
                    .ok()?
                    .to_string(),
            )
        } else {
            None
        }
    }
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
    thread::sleep(Duration::new(12, 0));
}
