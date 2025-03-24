use colored::Colorize;
use near_verify_rs::docker_command;
use std::io::IsTerminal;
use std::{
    process::{Command, ExitStatus},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "linux")]
use nix::unistd::{getgid, getuid};

use crate::env_keys;
use crate::pretty_print;
use crate::types::near::docker_build::cloned_repo;
use crate::types::near::docker_build::subprocess::container_paths;
use crate::types::near::docker_build::subprocess::env_vars;
use crate::types::near::docker_build::subprocess::env_vars::nep330_build_info::BuildInfoMixed;

/// TODO #F: set input params to be [near_verify_rs::types::nep330::ContractSourceMetadata] and `contract_sources_workdir`  of [std::path::PathBuf]
/// TODO: #F1: add [Vec<String>] `additional_docker_args` parameter
// TODO #E:  the `contract_source_workdir` is defined as `cloned_repo.tmp_repo_dir.path()`
/// TODO #H2: add validation of [BuildInfoMixed::build_environment] with `images_whitelist` [Vec<String>] argument
/// TODO #H1: check [BuildInfoMixed::build_environment] for regex match
pub fn run(
    build_info_mixed: BuildInfoMixed,
    cloned_repo: &cloned_repo::ClonedRepo,
) -> eyre::Result<(ExitStatus, Command)> {
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
            let pid = std::process::id().to_string();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string();
            format!("cargo-near-{}-{}", timestamp, pid)
        };
        let container_paths = container_paths::Paths::compute(cloned_repo)?;

        let docker_env_args = {
            let env = env_vars::EnvVars::new(build_info_mixed.clone())?;
            env.docker_args()
        };

        let shell_escaped_cargo_cmd = near_verify_rs::nep330::shell_escape_nep330_build_command(
            build_info_mixed.build_command,
        );
        println!(
            "{} {}",
            "build command in container:".green(),
            shell_escaped_cargo_cmd
        );
        println!();

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
            tracing::debug!("input device is a tty: {}", stdin_is_terminal);
            if stdin_is_terminal
                && std::env::var(env_keys::nep330::nonspec::SERVER_DISABLE_INTERACTIVE).is_err()
            {
                docker_args.push("-it");
            }

            docker_args.extend(docker_env_args.iter().map(|string| string.as_str()));

            docker_args.extend(vec![&build_info_mixed.build_environment, "/bin/bash", "-c"]);

            docker_args.push(&shell_escaped_cargo_cmd);
            docker_args
        };

        let mut docker_cmd = Command::new("docker");
        docker_cmd.arg("run");
        docker_cmd.args(docker_args);
        docker_cmd
    };
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Docker command:\n{}",
        pretty_print::indent_payload(&format!("{:#?}", docker_cmd))
    );

    let status_result = docker_cmd.status();
    let status = docker_command::handle_io_error(
        &docker_cmd,
        status_result,
        eyre::eyre!(super::ERR_REPRODUCIBLE),
    )?;

    Ok((status, docker_cmd))
}
