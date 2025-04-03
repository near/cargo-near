use colored::Colorize;
use near_verify_rs::logic::docker_checks;

use crate::docker::DockerBuildOpts;
use crate::pretty_print;
use crate::types::near::build::input::BuildContext;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::docker_build::subprocess::nep330_build_info::BuildInfoMixed;
use crate::types::near::docker_build::{cloned_repo, crate_in_repo, metadata};

pub mod git_checks;
pub mod warn_near_sdk_upgrades;

const RUST_LOG_EXPORT: &str = "RUST_LOG=info";

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
    warn_near_sdk_upgrades::suggest(cloned_repo.crate_metadata());

    let docker_build_meta = pretty_print::handle_step(
        &format!(
            "Parsing and validating `{}` section of contract's `Cargo.toml` ...",
            "[package.metadata.near.reproducible_build]".magenta()
        ),
        || metadata::ReproducibleBuild::parse(cloned_repo.crate_metadata()),
    )?;
    let contract_source_metadata = {
        let local_crate_info = BuildInfoMixed::new(&opts, &docker_build_meta, &cloned_repo)?;
        near_verify_rs::types::contract_source_metadata::ContractSourceMetadata::from(
            local_crate_info,
        )
    };

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
    if std::env::var(near_verify_rs::env_keys::nonspec::SERVER_DISABLE_INTERACTIVE).is_err() {
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

    contract_source_metadata.validate(None)?;
    let docker_build_out_wasm = near_verify_rs::logic::nep330_build::run(
        contract_source_metadata,
        cloned_repo.contract_source_workdir()?,
        additional_docker_args(),
    )?;

    cloned_repo.copy_artifact(docker_build_out_wasm, out_dir_arg)
}

fn additional_docker_args() -> Vec<String> {
    vec!["--env".to_string(), RUST_LOG_EXPORT.to_string()]
}
