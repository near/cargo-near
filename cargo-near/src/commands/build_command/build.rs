use cargo_near_build::camino::Utf8PathBuf;
use cargo_near_build::cargo_native;
use cargo_near_build::env_keys;
use cargo_near_build::near::abi;
use cargo_near_build::near_abi::BuildInfo;
use cargo_near_build::pretty_print;
use cargo_near_build::types::cargo::manifest_path::{ManifestPath, MANIFEST_FILE_NAME};
use cargo_near_build::types::cargo::metadata::CrateMetadata;
use cargo_near_build::types::near::abi as abi_types;
use cargo_near_build::types::near::build::version_mismatch::VersionMismatch;
use cargo_near_build::types::near::build::Opts;
use cargo_near_build::BuildArtifact;
use cargo_near_build::WASM;
use colored::Colorize;

use cargo_near_build::types::color_preference::ColorPreference;

use super::ArtifactMessages;

const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

pub fn run(args: Opts) -> color_eyre::eyre::Result<BuildArtifact> {
    VersionMismatch::export_builder_and_near_abi_versions();
    export_nep_330_build_command(&args)?;
    env_keys::nep330::print_env();

    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    pretty_print::handle_step("Checking the host environment...", || {
        if !cargo_native::target::wasm32_exists() {
            color_eyre::eyre::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
        }
        Ok(())
    })?;

    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        let manifest_path: Utf8PathBuf = if let Some(manifest_path) = args.manifest_path {
            manifest_path.into()
        } else {
            MANIFEST_FILE_NAME.into()
        };
        CrateMetadata::collect(ManifestPath::try_from(manifest_path)?, args.no_locked)
    })?;

    let out_dir = crate_metadata.resolve_output_dir(args.out_dir.map(Into::into))?;

    let mut build_env = vec![("RUSTFLAGS", "-C link-arg=-s")];
    let mut cargo_args = vec!["--target", COMPILATION_TARGET];
    let cargo_feature_args = {
        let mut feat_args = vec![];
        if let Some(features) = args.features.as_ref() {
            feat_args.extend(&["--features", features]);
        }

        if args.no_default_features {
            feat_args.push("--no-default-features");
        }
        feat_args
    };

    if !args.no_release {
        cargo_args.push("--release");
    }
    if !args.no_locked {
        cargo_args.push("--locked");
    }

    let mut abi = None;
    let mut min_abi_path = None;
    let (cargo_near_version, cargo_near_version_mismatch) =
        VersionMismatch::get_coerced_builder_version()?;
    if !args.no_abi {
        let mut contract_abi = abi::generate::procedure(
            &crate_metadata,
            args.no_locked,
            !args.no_doc,
            true,
            &cargo_feature_args,
            color.clone(),
        )?;

        contract_abi.metadata.build = Some(BuildInfo {
            compiler: format!("rustc {}", rustc_version::version()?),
            builder: format!("cargo-near {}", cargo_near_version),
            image: None,
        });
        if !args.no_embed_abi {
            let path = pretty_print::handle_step("Compressing ABI to be embedded..", || {
                let abi_types::Result { path } = abi::write_to_file(
                    &contract_abi,
                    &crate_metadata,
                    abi_types::Format::JsonMin,
                    abi_types::Compression::Zstd,
                )?;
                Ok(path)
            })?;
            min_abi_path.replace(cargo_near_build::fs::copy(&path, &out_dir)?);
        }
        abi = Some(contract_abi);
    }

    cargo_args.extend(cargo_feature_args);

    if let (false, Some(abi_path)) = (args.no_embed_abi, &min_abi_path) {
        cargo_args.extend(&["--features", "near-sdk/__abi-embed"]);
        build_env.push(("CARGO_NEAR_ABI_PATH", abi_path.as_str()));
    }

    let version = crate_metadata.root_package.version.to_string();
    build_env.push((env_keys::nep330::VERSION, &version));
    // this will be set in docker builds (externally to current process), having more info about git commit
    if std::env::var(env_keys::nep330::LINK).is_err() {
        if let Some(ref repository) = crate_metadata.root_package.repository {
            build_env.push((env_keys::nep330::LINK, repository));
        }
    }

    pretty_print::step("Building contract");
    let mut wasm_artifact = cargo_native::compile::run::<WASM>(
        &crate_metadata.manifest_path,
        &cargo_args,
        build_env,
        false,
        color,
    )?;

    wasm_artifact.path = cargo_near_build::fs::copy(&wasm_artifact.path, &out_dir)?;
    wasm_artifact.cargo_near_version_mismatch = cargo_near_version_mismatch;

    // todo! if we embedded, check that the binary exports the __contract_abi symbol

    pretty_print::success(&format!(
        "Contract successfully built! (in CARGO_NEAR_BUILD_ENVIRONMENT={})",
        std::env::var(env_keys::nep330::BUILD_ENVIRONMENT).unwrap_or("host".into())
    ));
    let mut messages = ArtifactMessages::default();
    messages.push_binary(&wasm_artifact)?;
    if let Some(mut abi) = abi {
        abi.metadata.wasm_hash = Some(wasm_artifact.compute_hash()?.to_base58_string());

        let abi_types::Result { path } = abi::write_to_file(
            &abi,
            &crate_metadata,
            abi_types::Format::Json,
            abi_types::Compression::NoOp,
        )?;
        let pretty_abi_path = cargo_near_build::fs::copy(&path, &out_dir)?;
        messages.push_free(("ABI", pretty_abi_path.to_string().yellow().bold()));
    }
    if let Some(abi_path) = min_abi_path {
        messages.push_free(("Embedded ABI", abi_path.to_string().yellow().bold()));
    }

    messages.pretty_print();
    Ok(wasm_artifact)
}

fn export_nep_330_build_command(args: &Opts) -> color_eyre::eyre::Result<()> {
    log::debug!(
        "compute `CARGO_NEAR_BUILD_COMMAND`,  current executable: {:?}",
        std::env::args().collect::<Vec<_>>()
    );
    let env_value: Vec<String> = match std::env::args().next() {
        // this is for cli context, being called from `cargo-near` bin
        Some(cli_arg_0)
            if cli_arg_0.ends_with("cargo-near") || cli_arg_0.ends_with("cargo-near.exe") =>
        {
            let mut cmd: Vec<String> = vec!["cargo".into()];
            // skipping `cargo-near`
            cmd.extend(std::env::args().skip(1));
            cmd
        }
        // this is for lib context, when build method is called from code
        // where `cargo-near` is an unlikely name to be chosen for executable
        _ => {
            // NOTE: order of output of cli flags shouldn't be too important, as the version of
            // `cargo-near` used as lib will be fixed in `Cargo.lock`
            args.get_cli_build_command()
        }
    };

    std::env::set_var(
        env_keys::nep330::BUILD_COMMAND,
        serde_json::to_string(&env_value)?,
    );
    Ok(())
}
