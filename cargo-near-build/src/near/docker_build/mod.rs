use colored::Colorize;
use near_verify_rs::logic::docker_checks;

use crate::docker::DockerBuildOpts;
use crate::pretty_print;
use crate::types::near::build::input::BuildContext;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::docker_build::subprocess::nep330_build_info::BuildInfoMixed;
use crate::types::near::docker_build::{cloned_repo, crate_in_repo, metadata};

pub mod git_checks;
pub mod warn_versions_upgrades;

const RUST_LOG_EXPORT: &str = "RUST_LOG=info";

pub fn run(opts: DockerBuildOpts, quiet: bool) -> eyre::Result<CompilationArtifact> {
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

    let applied_build_meta = {
        let docker_build_meta_parsed = pretty_print::handle_step(
            &format!(
                "Parsing `{}` section of contract's `Cargo.toml` ...",
                "[package.metadata.near.reproducible_build]".magenta()
            ),
            || metadata::parse::ReproducibleBuild::parse(cloned_repo.crate_metadata()),
        )?;

        let section_name = metadata::section_name(opts.variant.as_ref());
        pretty_print::handle_step(
            &format!(
                "Applying and validating `{}` section of contract's `Cargo.toml` ...",
                section_name.magenta()
            ),
            || {
                let applied_variant =
                    docker_build_meta_parsed.apply_variant_or_default(opts.variant.as_deref())?;
                applied_variant.validate()?;
                println!();
                println!(
                    "{} {}",
                    "applied reproducible build metadata:".green(),
                    applied_variant
                );
                Ok(applied_variant)
            },
        )?
    };

    let near_sdk_support =
        warn_versions_upgrades::suggest_near_sdk_checks(cloned_repo.crate_metadata());
    warn_versions_upgrades::suggest_cargo_near_build_checks(
        cloned_repo.crate_metadata(),
        &applied_build_meta,
        near_sdk_support,
    );
    let contract_source_metadata = {
        let local_crate_info = BuildInfoMixed::new(&opts, &applied_build_meta, &cloned_repo)?;
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
                        // this unwrap depends on `metadata::AppliedReproducibleBuild::validate` logic
                        &applied_build_meta.repository.clone().unwrap(),
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
            docker_checks::sanity::check(quiet)
        })?;

        pretty_print::handle_step("Checking that specified image is available...", || {
            let docker_image = applied_build_meta.concat_image();
            docker_checks::pull_image::check(&docker_image, quiet)
        })?;
    }

    pretty_print::step("Running build in docker command step...");
    let out_dir_arg = opts.out_dir.clone();

    contract_source_metadata.validate(None)?;
    let docker_build_out_wasm = near_verify_rs::logic::nep330_build::run(
        contract_source_metadata,
        cloned_repo.contract_source_workdir()?,
        additional_docker_args(&opts, &crate_in_repo.repo_root)?,
        quiet,
    )?;

    cloned_repo.copy_artifact(docker_build_out_wasm, out_dir_arg)
}

fn additional_docker_args(opts: &DockerBuildOpts, repo_root: &camino::Utf8Path) -> eyre::Result<Vec<String>> {
    let mut args = vec!["--env".to_string(), RUST_LOG_EXPORT.to_string()];
    
    // Mount ./target directory if requested
    if opts.mount_target_cache {
        let target_dir = repo_root.join("target");
        
        // Create target directory if it doesn't exist
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| eyre::eyre!("Failed to create target directory {:?}: {}", target_dir, e))?;
        
        let target_dir_str = dunce::canonicalize(&target_dir)
            .map_err(|e| eyre::eyre!("Failed to canonicalize target directory {:?}: {}", target_dir, e))?
            .to_string_lossy()
            .to_string();
        
        args.push("--volume".to_string());
        args.push(format!("{}:/near/target", target_dir_str));
        
        println!(
            "{}{}",
            "Mounting local target cache: ".cyan(),
            target_dir.as_str().yellow()
        );
    }
    
    // Mount ~/.cargo directory if requested
    if opts.mount_cargo_cache {
        let cargo_home = std::env::var("CARGO_HOME")
            .or_else(|_| {
                std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .map(|home| format!("{}/.cargo", home))
            })
            .map_err(|_| eyre::eyre!("Failed to determine CARGO_HOME or ~/.cargo directory"))?;
        
        let cargo_home_str = dunce::canonicalize(&cargo_home)
            .map_err(|e| eyre::eyre!("Failed to canonicalize cargo home {:?}: {}", cargo_home, e))?
            .to_string_lossy()
            .to_string();
        
        args.push("--volume".to_string());
        args.push(format!("{}:/cargo_cache", cargo_home_str));
        
        println!(
            "{}{}",
            "Mounting local cargo cache: ".cyan(),
            cargo_home.yellow()
        );
    }
    
    Ok(args)
}
