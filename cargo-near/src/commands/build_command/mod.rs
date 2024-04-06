use std::process::{Command, id};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use nix::unistd::{getuid, getgid};

use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    owo_colors::OwoColorize,
};
use env_logger::fmt::Timestamp;

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

    let tmp_contract_dir = tempfile::tempdir()?;
    let mut tmp_contract_path = tmp_contract_dir.path().to_path_buf();

    let tmp_repo = git2::Repository::clone(contract_path.as_str(), &tmp_contract_path)?;

    // Cross-platform process ID and timestamp
    let pid = id().to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();

    let volume = format!(
        "{}:/host",
        tmp_repo
            .workdir()
            .wrap_err("Could not get the working directory for the repository")?
            .to_string_lossy()
    );
    let docker_image = "docker.io/sourcescan/cargo-near:0.6.0-builder"; //XXX need to fix version!!! image from cargo.toml for contract
    let docker_container_name = format!("cargo-near-{}-{}", timestamp, pid);
    let near_build_env_ref = format!("NEAR_BUILD_ENVIRONMENT_REF={}", docker_image);

    // Platform-specific UID/GID retrieval
    #[cfg(unix)]
    let uid_gid = format!("{}:{}", getuid(), getgid());
    #[cfg(not(unix))]
    let uid_gid = "1000:1000".to_string();

    let mut docker_args = vec![
        "-u", &uid_gid,
        "-it",
        "--name", &docker_container_name,
        "--volume", &volume,
        "--rm",
        "--workdir", "/host",
        "--env", &near_build_env_ref,
        docker_image,
        "/bin/bash", "-c"
    ];

    let mut cargo_cmd_list = vec![
        "cargo",
        "near",
        "build",
    ];
    cargo_cmd_list.extend(&cargo_args);

    let cargo_cmd = cargo_cmd_list.join(" ");
    
    docker_args.push(&cargo_cmd);

    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    docker_cmd.args(docker_args);

    let status = match docker_cmd.status() {
        Ok(exit_status) => exit_status,
        Err(io_err) => {
            println!("Error obtaining status from executing SourceScan command `{:?}`", docker_cmd);
            println!("Error `{:?}`", io_err);
            return Err(color_eyre::eyre::eyre!(
                "Reproducible build in docker container failed"
            ))
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
                std::fs::copy::<std::path::PathBuf, camino::Utf8PathBuf>(
                    entry.path(),
                    contract_path.clone(),
                )?;

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

        Err(color_eyre::eyre::eyre!(
            "Reproducible build in docker container failed"
        ))
    }
}
