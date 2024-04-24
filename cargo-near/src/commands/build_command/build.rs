use camino::Utf8PathBuf;
use colored::Colorize;
use near_abi::BuildInfo;
use sha2::{Digest, Sha256};

use crate::commands::abi_command::abi::{AbiCompression, AbiFormat, AbiResult};
use crate::commands::build_command::BUILD_CMD_ENV_KEY;
use crate::common::ColorPreference;
use crate::types::manifest::MANIFEST_FILE_NAME;
use crate::types::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::util;
use crate::{commands::abi_command::abi, util::wasm32_target_libdir_exists};

use super::ArtifactMessages;

const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

pub(super) fn run(
    args: super::BuildCommand,
) -> color_eyre::eyre::Result<util::CompilationArtifact> {
    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    export_nep_330_build_command();

    util::handle_step("Checking the host environment...", || {
        if !wasm32_target_libdir_exists() {
            color_eyre::eyre::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
        }
        Ok(())
    })?;

    let crate_metadata = util::handle_step("Collecting cargo project metadata...", || {
        let manifest_path: Utf8PathBuf = if let Some(manifest_path) = args.manifest_path {
            manifest_path.into()
        } else {
            MANIFEST_FILE_NAME.into()
        };
        CrateMetadata::collect(CargoManifestPath::try_from(manifest_path)?, args.no_locked).map_err(|err| {
            if !args.no_locked && err.to_string().contains("Cargo.lock is absent") {
                println!(
                    "{}",
                    " You can choose to disable `--locked` flag for downstream `cargo` command with `--no-locked` flag.".cyan()
                );
            }
            err
        })
    })?;

    let out_dir = crate_metadata.resolve_output_dir(args.out_dir)?;

    let mut build_env = vec![("RUSTFLAGS", "-C link-arg=-s")];
    let mut cargo_args = vec!["--target", COMPILATION_TARGET];
    if !args.no_release {
        cargo_args.push("--release");
    }
    if !args.no_locked {
        cargo_args.push("--locked");
    }

    let mut abi = None;
    let mut min_abi_path = None;
    if !args.no_abi {
        let mut contract_abi = abi::generate_abi(
            &crate_metadata,
            args.no_locked,
            !args.no_doc,
            true,
            color.clone(),
        )?;
        contract_abi.metadata.build = Some(BuildInfo {
            compiler: format!("rustc {}", rustc_version::version()?),
            builder: format!("cargo-near {}", env!("CARGO_PKG_VERSION")),
            image: None,
        });
        if !args.no_embed_abi {
            let path = util::handle_step("Compressing ABI to be embedded..", || {
                let AbiResult { path } = abi::write_to_file(
                    &contract_abi,
                    &crate_metadata,
                    AbiFormat::JsonMin,
                    AbiCompression::Zstd,
                )?;
                Ok(path)
            })?;
            min_abi_path.replace(util::copy(&path, &out_dir)?);
        }
        abi = Some(contract_abi);
    }

    if let (false, Some(abi_path)) = (args.no_embed_abi, &min_abi_path) {
        cargo_args.extend(&["--features", "near-sdk/__abi-embed"]);
        build_env.push(("CARGO_NEAR_ABI_PATH", abi_path.as_str()));
    }
    util::print_step("Building contract");
    let mut wasm_artifact = util::compile_project(
        &crate_metadata.manifest_path,
        &cargo_args,
        build_env,
        "wasm",
        false,
        color,
    )?;

    wasm_artifact.path = util::copy(&wasm_artifact.path, &out_dir)?;

    // todo! if we embedded, check that the binary exports the __contract_abi symbol
    util::print_success("Contract successfully built!");
    let mut messages = ArtifactMessages::default();
    messages.push_binary(&wasm_artifact);
    if let Some(mut abi) = abi {
        let mut hasher = Sha256::new();
        hasher.update(std::fs::read(&wasm_artifact.path)?);
        let hash = hasher.finalize();
        let hash = bs58::encode(hash).into_string();
        abi.metadata.wasm_hash = Some(hash);

        let AbiResult { path } =
            abi::write_to_file(&abi, &crate_metadata, AbiFormat::Json, AbiCompression::NoOp)?;
        let pretty_abi_path = util::copy(&path, &out_dir)?;
        messages.push_free(("ABI", pretty_abi_path.to_string().yellow().bold()));
    }
    if let Some(abi_path) = min_abi_path {
        messages.push_free(("Embedded ABI", abi_path.to_string().yellow().bold()));
    }

    messages.pretty_print();
    Ok(wasm_artifact)
}

fn export_nep_330_build_command() {
    let mut cmd: Vec<String> = vec!["cargo".into(), "near".into()];
    cmd.extend(std::env::args().skip(2));

    let cmd = cmd.join(" ");

    std::env::set_var(BUILD_CMD_ENV_KEY, cmd.clone());
    log::info!("exported: {}='{}'", BUILD_CMD_ENV_KEY, cmd);
}
