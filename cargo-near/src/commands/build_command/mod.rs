use std::process::Command;

use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    owo_colors::OwoColorize,
};
use serde_json::to_string;

pub mod build;

#[derive(Debug, Default, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = BuildCommandlContext)]
pub struct BuildCommand {
    /// Build contract without SourceScan verification
    #[interactive_clap(long)]
    pub no_docker: bool,
    /// Build contract in debug mode, without optimizations and bigger is size
    #[interactive_clap(long)]
    pub no_release: bool,
    /// Do not generate ABI for the contract
    #[interactive_clap(long)]
    pub no_abi: bool,
    /// Do not embed the ABI in the contract binary
    #[interactive_clap(long)]
    pub no_embed_abi: bool,
    /// Do not include rustdocs in the embedded ABI
    #[interactive_clap(long)]
    pub no_doc: bool,
    /// Copy final artifacts to this directory
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Coloring: auto, always, never
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_interactive_input)]
    pub color: Option<crate::common::ColorPreference>,
}

#[derive(Debug, Clone)]
pub struct BuildCommandlContext;

impl BuildCommandlContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        scope: &<BuildCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let args = BuildCommand {
            no_docker: scope.no_docker,
            no_release: scope.no_release,
            no_abi: scope.no_abi,
            no_embed_abi: scope.no_embed_abi,
            no_doc: scope.no_doc,
            out_dir: scope.out_dir.clone(),
            manifest_path: scope.manifest_path.clone(),
            color: scope.color.clone(),
        };
        if args.no_docker {
            self::build::run(args)?;
        } else {
            docker_run(args)?;
        }
        Ok(Self)
    }
}

pub fn docker_run(args: BuildCommand) -> color_eyre::eyre::Result<camino::Utf8PathBuf> {
    let mut cargo_args = vec![];
    // Use this in new release version:
    // let mut cargo_args = vec!["--no-docker"];

    if args.no_abi {
        cargo_args.push("--no-abi")
    }
    if args.no_embed_abi {
        cargo_args.push("--no-embed-abi")
    }
    if args.no_doc {
        cargo_args.push("--no-doc")
    }
    let color = args
        .color
        .clone()
        .unwrap_or(crate::common::ColorPreference::Auto)
        .to_string();
    cargo_args.extend(&["--color", &color]);

    let mut contract_path: camino::Utf8PathBuf = if let Some(manifest_path) = &args.manifest_path {
        manifest_path.into()
    } else {
        camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?).map_err(|err| {
            color_eyre::eyre::eyre!("Failed to convert path {}", err.to_string_lossy())
        })?
    };

    let repo_id = check_repo_state(&contract_path)?;
    println!("repo_id: {repo_id}");

    let tmp_contract_dir = tempfile::tempdir()?;
    let mut tmp_contract_path = tmp_contract_dir.path().to_path_buf();

    let tmp_repo = git2::Repository::clone(contract_path.as_str(), &tmp_contract_path)?;

    let volume = format!(
        "{}:/host",
        tmp_repo
            .workdir()
            .wrap_err("Could not get the working directory for the repository")?
            .to_string_lossy()
    );
    let mut docker_args = vec![
        "--name",
        "cargo-near-container",
        "--volume",
        &volume,
        "--rm",
        "--workdir",
        "/host",
        "--env",
        "NEAR_BUILD_ENVIRONMENT_REF=docker.io/sourcescan/cargo-near:0.6.0",
        "docker.io/sourcescan/cargo-near:0.6.0", //XXX need to fix version!!! image from cargo.toml for contract
        "cargo",
        "near",
        "build",
    ];
    docker_args.extend(&cargo_args);

    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    docker_cmd.args(docker_args);

    let status = match docker_cmd.status() {
        Ok(exit_status) => exit_status,
        Err(_) => {
            println!("Error executing SourceScan command `{:?}`", docker_cmd);
            println!(
                "{}",
                "WARNING! Compilation without SourceScan verification".red()
            );
            return Ok(self::build::run(args)?.path);
        }
    };

    if status.success() {
        tmp_contract_path.push("target");
        tmp_contract_path.push("near");

        let dir = tmp_contract_path
            .read_dir()
            .wrap_err_with(|| format!("No artifacts directory found: `{tmp_contract_path:?}`."))?;

        for entry in dir.flatten() {
            if entry.path().extension().unwrap().to_str().unwrap() == "wasm" {
                contract_path.push("contract.wasm");
                let _ = std::fs::rename::<std::path::PathBuf, camino::Utf8PathBuf>(
                    entry.path(),
                    contract_path.clone(),
                );
                return Ok(contract_path);
            }
        }

        Err(color_eyre::eyre::eyre!(
            "Wasm file not found in directory: `{tmp_contract_path:?}`."
        ))
    } else {
        println!(
            "SourceScan command `{:?}` failed with exit status: {status}",
            docker_cmd
        );
        println!(
            "{}",
            "WARNING! Compilation without SourceScan verification".red()
        );
        Ok(self::build::run(args)?.path)
    }
}

fn check_repo_state(contract_path: &camino::Utf8PathBuf) -> color_eyre::Result<git2::Oid> {
    let repo = git2::Repository::open(contract_path)?;

    let mut dirty_files = Vec::new();
    collect_statuses(&repo, &mut dirty_files)?;
    // Include each submodule so that the error message can provide
    // specifically *which* files in a submodule are modified.
    status_submodules(&repo, &mut dirty_files)?;

    if dirty_files.is_empty() {
        Ok(repo.revparse_single("HEAD")?.id())
    } else {
        color_eyre::eyre::bail!(
            "{} files in the working directory contain changes that were \
             not yet committed into git:\n\n{}\n\n\
             commit these changes to continue",
            dirty_files.len(),
            dirty_files
                .iter()
                .map(to_string)
                .collect::<Result<Vec<_>, _>>()
                .wrap_err("Error parsing PathBaf")?
                .join("\n")
        )
    }
}

// Helper to collect dirty statuses for a single repo.
fn collect_statuses(
    repo: &git2::Repository,
    dirty_files: &mut Vec<std::path::PathBuf>,
) -> near_cli_rs::CliResult {
    let mut status_opts = git2::StatusOptions::new();
    // Exclude submodules, as they are being handled manually by recursing
    // into each one so that details about specific files can be
    // retrieved.
    status_opts
        .exclude_submodules(true)
        .include_ignored(true)
        .include_untracked(true);
    let repo_statuses = repo.statuses(Some(&mut status_opts)).with_context(|| {
        format!(
            "failed to retrieve git status from repo {}",
            repo.path().display()
        )
    })?;
    let workdir = repo.workdir().unwrap();
    let this_dirty = repo_statuses.iter().filter_map(|entry| {
        let path = entry.path().expect("valid utf-8 path");
        if path.ends_with("Cargo.lock") || entry.status() == git2::Status::IGNORED {
            return None;
        }
        Some(workdir.join(path))
    });
    dirty_files.extend(this_dirty);
    Ok(())
}

// Helper to collect dirty statuses while recursing into submodules.
fn status_submodules(
    repo: &git2::Repository,
    dirty_files: &mut Vec<std::path::PathBuf>,
) -> near_cli_rs::CliResult {
    for submodule in repo.submodules()? {
        // Ignore submodules that don't open, they are probably not initialized.
        // If its files are required, then the verification step should fail.
        if let Ok(sub_repo) = submodule.open() {
            status_submodules(&sub_repo, dirty_files)?;
            collect_statuses(&sub_repo, dirty_files)?;
        }
    }
    Ok(())
}
