use cargo_near_build::docker_build_types::WARN_BECOMES_ERR;
use cargo_near_build::docker_build_types::{
    cloned_repo, container_paths, crate_in_repo, env_vars, metadata,
};
use cargo_near_build::{camino, BuildContext, BuildOpts, DockerBuildOpts};
use cargo_near_build::{env_keys, pretty_print, BuildArtifact};
use std::time::Duration;
use std::{
    io::IsTerminal,
    process::{id, Command, ExitStatus},
    time::{SystemTime, UNIX_EPOCH},
};

use colored::Colorize;
#[cfg(target_os = "linux")]
use nix::unistd::{getgid, getuid};

mod docker_checks;
mod git_checks;

const ERR_REPRODUCIBLE: &str = "Reproducible build in docker container failed.";

pub(super) fn docker_run(docker_opts: DockerBuildOpts) -> color_eyre::eyre::Result<BuildArtifact> {
    let opts = docker_opts.build_opts;
    let color = opts
        .color
        .clone()
        .unwrap_or(cargo_near_build::ColorPreference::Auto);
    color.apply();
    let crate_in_repo = pretty_print::handle_step(
        "Opening repo and determining HEAD and relative path of contract...",
        || crate_in_repo::Crate::find(&opts.contract_path()?),
    )?;
    pretty_print::handle_step("Checking if git is dirty...", || {
        git_dirty_check(docker_opts.context, &crate_in_repo.repo_root)
    })?;
    let cloned_repo = pretty_print::handle_step(
        "Cloning project repo to a temporary build site, removing uncommitted changes...",
        || {
            cloned_repo::ClonedRepo::check_locked_then_clone(
                crate_in_repo.clone(),
                opts.no_locked,
                docker_opts.context,
            )
        },
    )?;

    let docker_build_meta =
        pretty_print::handle_step("Parsing and validating `Cargo.toml` metadata...", || {
            metadata::ReproducibleBuild::parse(cloned_repo.crate_metadata())
        })?;

    if let BuildContext::Deploy = docker_opts.context {
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
    let out_dir_arg = opts.out_dir.clone();
    let (status, docker_cmd) = docker_run_subprocess_step(opts, docker_build_meta, &cloned_repo)?;

    handle_docker_run_status(status, docker_cmd, cloned_repo, out_dir_arg)
}

fn docker_run_subprocess_step(
    opts: BuildOpts,
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
        let container_paths = container_paths::Paths::compute(cloned_repo)?;
        let docker_image = docker_build_meta.concat_image();

        let env = env_vars::EnvVars::new(&docker_build_meta, cloned_repo)?;
        let env_args = env.docker_args();
        let cargo_cmd = opts
            .get_cli_build_command_in_docker(docker_build_meta.container_build_command.clone())?;
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
    out_dir_arg: Option<camino::Utf8PathBuf>,
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
            std::thread::sleep(Duration::new(3, 0));
            println!();
            println!("{}", WARN_BECOMES_ERR.red(),);
            // this is magic to help user notice:
            std::thread::sleep(Duration::new(5, 0));

            Ok(())
        }
        _ => Ok(()),
    }
}
