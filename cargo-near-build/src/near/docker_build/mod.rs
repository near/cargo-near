use std::process::{Command, ExitStatus};

use crate::types::near::build::input::BuildContext;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::docker_build::{cloned_repo, crate_in_repo, metadata};
use crate::{docker::DockerBuildOpts, env_keys, pretty_print};

pub mod docker_checks;
pub mod git_checks;
pub mod subprocess_step;

pub const ERR_REPRODUCIBLE: &str = "Reproducible build in docker container failed.";

pub fn run(docker_opts: DockerBuildOpts) -> eyre::Result<CompilationArtifact> {
    let opts = docker_opts.build_opts;
    let color = opts.color.clone().unwrap_or(crate::ColorPreference::Auto);
    color.apply();
    let crate_in_repo = pretty_print::handle_step(
        "Opening repo and determining HEAD and relative path of contract...",
        || crate_in_repo::Crate::find(&opts.contract_path()?),
    )?;
    pretty_print::handle_step("Checking if git is dirty...", || {
        git_checks::dirty::check_then_handle(docker_opts.context, &crate_in_repo.repo_root)
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
    let (status, docker_cmd) = subprocess_step::run(opts, docker_build_meta, &cloned_repo)?;

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
        docker_checks::print_command_status(status, command);
        Err(eyre::eyre!(ERR_REPRODUCIBLE))
    }
}
