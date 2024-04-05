use std::process::Command;

use camino::Utf8PathBuf;
use color_eyre::{eyre::{ContextCompat, WrapErr}, owo_colors::OwoColorize};
use serde::Deserialize;

use crate::{types::{manifest::CargoManifestPath, metadata::CrateMetadata}, util};

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

#[derive(Deserialize, Debug)]
struct ReproducibleBuildMeta{
    build_environment: String,
}

fn get_metadata(args: &BuildCommand) -> color_eyre::eyre::Result<CrateMetadata> {
    let crate_metadata = util::handle_step("Collecting cargo project metadata...", || {
        let manifest_path: Utf8PathBuf = if let Some(manifest_path) = &args.manifest_path {
            manifest_path.into()
        } else {
            "Cargo.toml".into()
        };
        log::trace!("crate manifest path : {:?}", manifest_path);
        CrateMetadata::collect(CargoManifestPath::try_from(manifest_path)?)
    })?;
    log::trace!("crate metadata : {:#?}", crate_metadata);
    Ok(crate_metadata)
    
}

fn get_docker_build_meta(cargo_metadata: &CrateMetadata) -> color_eyre::eyre::Result<ReproducibleBuildMeta>{
    let build_meta_value = cargo_metadata.root_package.metadata.get(
        "near"
            ).and_then( |value | value.get("reproducible_build"));

    let build_meta: ReproducibleBuildMeta = match build_meta_value {
        None => return Err(color_eyre::eyre::eyre!("Missing reproducible build metadata `[package.metadata.near.reproducible_build]` in Cargo.toml")), 

        Some(build_meta_value) => {
           serde_json::from_value(build_meta_value.clone()).map_err(|err| {
           
            color_eyre::eyre::eyre!("Malformed reproducible build metadata `[package.metadata.near.reproducible_build]`: {}", err)
           })?
        }
    };
    log::info!("reproducible build metadata: {:#?}", build_meta);
    Ok(build_meta)
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

    let cargo_metadata = get_metadata(&args)?;
    let docker_build_meta = get_docker_build_meta(&cargo_metadata)?;


    let mut contract_path: camino::Utf8PathBuf = if let Some(manifest_path) = &args.manifest_path {
        manifest_path.into()
    } else {
        camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?).map_err(|err| {
            color_eyre::eyre::eyre!("Failed to convert path {}", err.to_string_lossy())
        })?
    };

    let tmp_contract_dir = tempfile::tempdir()?;
    log::debug!("temporary contract dir : {:?}", tmp_contract_dir);
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
        "-it",
        "--name",
        "cargo-near-container",
        "--volume",
        &volume,
        "--rm",
        "--workdir",
        "/host",
        "--env",
        "NEAR_BUILD_ENVIRONMENT_REF=docker.io/sourcescan/cargo-near:0.6.0",
        &docker_build_meta.build_environment,
        "sh",
        "-c"
    ];
    let mut cargo_cmd_list = vec![
        "cargo",
        "near",
        "build",
    ];
    cargo_cmd_list.extend(&cargo_args);

    let cargo_cmd = cargo_cmd_list.join(" ");
    
    docker_args.push(&cargo_cmd);

    log::debug!("docker command : {:?}", docker_args);

    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    docker_cmd.args(docker_args);

    let status = match docker_cmd.status() {
        Ok(exit_status) => exit_status,
        Err(io_err) => {
            println!();
            println!("{}", format!("Error obtaining status from executing SourceScan command `{:?}`", docker_cmd).yellow());
            println!("{}", format!("Error `{:?}`", io_err).yellow());
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
        println!();
        println!(
            "{}",
            format!(
                "See output above ↑↑↑.\nSourceScan command `{:?}` failed with exit status: {status}.",
                docker_cmd
            ).yellow()
        );

        Err(color_eyre::eyre::eyre!(
            "Reproducible build in docker container failed"
        ))
    }
}
