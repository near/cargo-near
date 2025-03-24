use std::process::{Command, ExitStatus};

use colored::Colorize;
use near_verify_rs::{docker_checks, docker_command};

use crate::docker::DockerBuildOpts;
use crate::types::near::build::input::BuildContext;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::docker_build::subprocess::env_vars::nep330_build_info::BuildInfoMixed;
use crate::types::near::docker_build::{cloned_repo, crate_in_repo, metadata};
use crate::{env_keys, pretty_print};

pub mod git_checks;
pub mod subprocess_step;

pub const ERR_REPRODUCIBLE: &str = "Reproducible build in docker container failed.";

pub fn run(opts: DockerBuildOpts) -> eyre::Result<CompilationArtifact> {
    let color = opts.color.unwrap_or(crate::ColorPreference::Auto);
    color.apply();
    let crate_in_repo = pretty_print::handle_step(
        "Opening repo and determining HEAD and relative path of contract...",
        || crate_in_repo::Crate::find(&opts.contract_path()?),
    )?;
    pretty_print::handle_step("Checking if git is dirty...", || {
        git_checks::dirty::check_then_handle(opts.context, &crate_in_repo.repo_root)
    })?;
    let cloned_repo = pretty_print::handle_step(
        "Cloning project repo to a temporary build site, removing uncommitted changes...",
        || {
            cloned_repo::ClonedRepo::check_locked_then_clone(
                crate_in_repo.clone(),
                opts.no_locked,
                opts.context,
            )
        },
    )?;

    let docker_build_meta = pretty_print::handle_step(
        &format!(
            "Parsing and validating `{}` section of contract's `Cargo.toml` ...",
            "[package.metadata.near.reproducible_build]".magenta()
        ),
        || metadata::ReproducibleBuild::parse(cloned_repo.crate_metadata()),
    )?;

    if let BuildContext::Deploy {
        skip_git_remote_check,
    } = opts.context
    {
        if !skip_git_remote_check {
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
        } else {
            pretty_print::handle_step(
                "Check that current HEAD has been pushed to remote was configured out by `--skip-git-remote-check` flag",
                || {
                    Ok(())
                },
            )?;
        }
    }
    if std::env::var(env_keys::nep330::nonspec::SERVER_DISABLE_INTERACTIVE).is_err() {
        pretty_print::handle_step("Performing `docker` sanity check...", || {
            docker_checks::sanity::check()
        })?;

        pretty_print::handle_step("Checking that specified image is available...", || {
            let docker_image = docker_build_meta.concat_image();
            docker_checks::pull_image::check(&docker_image)
        })?;
    }

    pretty_print::step("Running build in docker command step...");
    let out_dir_arg = opts.out_dir.clone();
    let build_info_mixed = BuildInfoMixed::new(opts, &docker_build_meta, &cloned_repo)?;
    // TODO #F2: add `additional_docker_args` usage here from TODO: #F3
    let (status, docker_cmd) = subprocess_step::run(
        build_info_mixed,
        &cloned_repo,
        cloned_repo.contract_source_workdir()?,
    )?;

    handle_docker_run_status(status, docker_cmd, cloned_repo, out_dir_arg)
}

fn handle_docker_run_status(
    status: ExitStatus,
    command: Command,
    cloned_repo: cloned_repo::ClonedRepo,
    out_dir_arg: Option<camino::Utf8PathBuf>,
) -> eyre::Result<CompilationArtifact> {
    if status.success() {
        pretty_print::success("Running docker command step (finished)");
        pretty_print::handle_step("Copying artifact from temporary build site...", || {
            cloned_repo.copy_artifact(out_dir_arg)
        })
    } else {
        docker_command::print::command_status(status, command);
        Err(eyre::eyre!(ERR_REPRODUCIBLE))
    }
}
