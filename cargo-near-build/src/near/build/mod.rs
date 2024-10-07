use crate::cargo_native::Wasm;
use crate::types::near::abi as abi_types;
use camino::Utf8PathBuf;
use colored::Colorize;
use near_abi::BuildInfo;

use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::build::side_effects::ArtifactMessages;
use crate::{cargo_native, env_keys, ColorPreference};
use crate::{
    cargo_native::target::COMPILATION_TARGET,
    pretty_print,
    types::{
        cargo::{
            manifest_path::{ManifestPath, MANIFEST_FILE_NAME},
            metadata::CrateMetadata,
        },
        near::build::{input::Opts, output::version_mismatch::VersionMismatch},
    },
};

use super::abi;

pub mod export;

/// builds a contract whose crate root is current workdir, or identified by [`Cargo.toml`/BuildOpts::manifest_path](crate::BuildOpts::manifest_path) location
pub fn run(args: Opts) -> eyre::Result<CompilationArtifact> {
    VersionMismatch::export_builder_and_near_abi_versions();
    export::nep_330_build_command(&args)?;
    env_keys::nep330::print_env();

    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    pretty_print::handle_step("Checking the host environment...", || {
        if !cargo_native::target::wasm32_exists() {
            eyre::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
        }
        Ok(())
    })?;

    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        let manifest_path: Utf8PathBuf = if let Some(manifest_path) = args.manifest_path {
            manifest_path
        } else {
            MANIFEST_FILE_NAME.into()
        };
        CrateMetadata::collect(ManifestPath::try_from(manifest_path)?, args.no_locked)
    })?;

    let out_dir = crate_metadata.resolve_output_dir(args.out_dir.map(Into::into))?;

    let mut build_env;
    if rustc_version::version().unwrap() >= rustc_version::Version::parse("1.82.0").unwrap() {
        build_env = vec![(
            "RUSTFLAGS",
            "-C link-arg=-s -C target-feature=-multivalue,-reference-types",
        )];
    } else {
        build_env = vec![("RUSTFLAGS", "-C link-arg=-s")];
    }

    build_env.extend(
        args.env
            .iter()
            .map(|(key, value)| (key.as_ref(), value.as_ref())),
    );
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
    let (builder_version, builder_version_mismatch) =
        VersionMismatch::get_coerced_builder_version()?;
    if !args.no_abi {
        let mut contract_abi = {
            let env = args
                .env
                .iter()
                .map(|(key, value)| (key.as_ref(), value.as_ref()))
                .collect::<Vec<_>>();
            abi::generate::procedure(
                &crate_metadata,
                args.no_locked,
                !args.no_doc,
                true,
                &cargo_feature_args,
                &env,
                color.clone(),
            )?
        };

        let embedding_binary = args.cli_description.cli_name_abi;
        contract_abi.metadata.build = Some(BuildInfo {
            compiler: format!("rustc {}", rustc_version::version()?),
            builder: format!("{} {}", embedding_binary, builder_version),
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
            min_abi_path.replace(crate::fs::copy(&path, &out_dir)?);
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
    let mut wasm_artifact = cargo_native::compile::run::<Wasm>(
        &crate_metadata.manifest_path,
        &cargo_args,
        build_env,
        false,
        color,
    )?;

    wasm_artifact.path = crate::fs::copy(&wasm_artifact.path, &out_dir)?;
    wasm_artifact.builder_version_mismatch = builder_version_mismatch;

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
        let pretty_abi_path = crate::fs::copy(&path, &out_dir)?;
        messages.push_free(("ABI", pretty_abi_path.to_string().yellow().bold()));
    }
    if let Some(abi_path) = min_abi_path {
        messages.push_free(("Embedded ABI", abi_path.to_string().yellow().bold()));
    }

    messages.pretty_print();
    Ok(wasm_artifact)
}
