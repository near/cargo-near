use super::docker_checks;
use crate::docker::DockerBuildOpts;
use colored::Colorize;
use std::io::IsTerminal;
use std::{
    process::{Command, ExitStatus},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "linux")]
use nix::unistd::{getgid, getuid};

use crate::env_keys;
use crate::pretty_print;
use crate::types::near::docker_build::subprocess::{container_paths, env_vars};
use crate::types::near::docker_build::{cloned_repo, metadata};

pub fn run(
    opts: DockerBuildOpts,
    docker_build_meta: metadata::ReproducibleBuild,
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
        let docker_image = docker_build_meta.concat_image();

        let env = env_vars::EnvVars::new(&docker_build_meta, cloned_repo)?;
        let env_args = env.docker_args();
        let shell_escaped_cargo_cmd = {
            let cargo_cmd = opts.get_cli_build_command_in_docker(&docker_build_meta)?;
            tracing::debug!("cli_build_command_in_docker {:#?}", cargo_cmd);
            shell_words::join(cargo_cmd)
        };
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

            docker_args.extend(env_args.iter().map(|string| string.as_str()));

            docker_args.extend(vec![&docker_image, "/bin/bash", "-c"]);

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
    let status = docker_checks::handle_command_io_error(
        &docker_cmd,
        status_result,
        eyre::eyre!(super::ERR_REPRODUCIBLE),
    )?;

    Ok((status, docker_cmd))
}
