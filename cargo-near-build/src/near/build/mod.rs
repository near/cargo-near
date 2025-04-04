use crate::cargo_native::Wasm;
use crate::types::near::abi as abi_types;
use crate::types::near::build::buildtime_env;
use camino::Utf8PathBuf;
use colored::Colorize;
use near_abi::BuildInfo;
use tempfile::NamedTempFile;

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
        near::build::{input::Opts, output::version_info::VersionInfo},
    },
};

use super::abi;

/// builds a contract whose crate root is current workdir, or identified by [`Cargo.toml`/BuildOpts::manifest_path](crate::BuildOpts::manifest_path) location
pub fn run(args: Opts) -> eyre::Result<CompilationArtifact> {
    let start = std::time::Instant::now();
    let override_cargo_target_path_env =
        buildtime_env::CargoTargetDir::maybe_new(args.override_cargo_target_dir.clone());

    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    pretty_print::handle_step("Checking the host environment...", || {
        if !cargo_native::target::wasm32_exists() {
            eyre::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
        }
        Ok(())
    })?;

    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        let manifest_path: Utf8PathBuf = if let Some(manifest_path) = args.manifest_path.clone() {
            manifest_path
        } else {
            MANIFEST_FILE_NAME.into()
        };
        let manifest_path = ManifestPath::try_from(manifest_path)?;
        CrateMetadata::collect(
            manifest_path,
            args.no_locked,
            override_cargo_target_path_env.as_ref(),
        )
    })?;

    // addition of this check wasn't a change in logic, as previously output path was
    // assumed without `--out-dir` too, so docker-build was just failing if the arg was supplied:
    // https://github.com/near/cargo-near/blob/075d7b6dc9ab1f5c199edb6931512ccaf5af848e/cargo-near-build/src/types/near/docker_build/cloned_repo.rs#L100
    if env_keys::is_inside_docker_context() && args.out_dir.is_some() {
        return Err(eyre::eyre!("inside docker build `--out-dir` is forbidden to be used in order to predict build output path in a straightforward way"));
    }
    // NOTE important!: the way the output path for wasm is resolved now cannot change,
    // see more detail on [CrateMetadata::get_legacy_cargo_near_output_path]
    let output_paths = crate_metadata.get_legacy_cargo_near_output_path(args.out_dir.clone())?;

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
    let builder_version_info = VersionInfo::get_coerced_builder_version()?;

    let common_vars_env = buildtime_env::CommonVariables::new(
        &args,
        &builder_version_info,
        &crate_metadata,
        override_cargo_target_path_env,
        &output_paths,
    )?;
    env_keys::print_nep330_env(
        &common_vars_env.nep330_output_wasm_path,
        &common_vars_env.nep330_link,
    );

    if !args.no_abi {
        let mut contract_abi = {
            let mut abi_env = args
                .env
                .iter()
                .map(|(key, value)| (key.as_ref(), value.as_ref()))
                .collect::<Vec<_>>();
            common_vars_env.append_borrowed_to(&mut abi_env);

            abi::generate::procedure(
                &crate_metadata,
                args.no_locked,
                !args.no_doc,
                true,
                &cargo_feature_args,
                &abi_env,
                color,
            )?
        };

        let embedding_binary = args.cli_description.cli_name_abi;
        contract_abi.metadata.build = Some(BuildInfo {
            compiler: format!("rustc {}", rustc_version::version()?),
            builder: format!(
                "{} {}",
                embedding_binary,
                builder_version_info.result_builder_version()?
            ),
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
            min_abi_path.replace(crate::fs::copy(&path, &output_paths.out_dir)?);
        }
        abi = Some(contract_abi);
    }

    cargo_args.extend(cargo_feature_args);

    if let (false, Some(..)) = (args.no_embed_abi, &min_abi_path) {
        cargo_args.extend(&["--features", "near-sdk/__abi-embed"]);
    }

    let abi_path_env = buildtime_env::AbiPath::new(args.no_embed_abi, &min_abi_path);

    let build_env = {
        let mut build_env = vec![(env_keys::RUSTFLAGS, "-C link-arg=-s")];
        build_env.extend(
            args.env
                .iter()
                .map(|(key, value)| (key.as_ref(), value.as_ref())),
        );

        abi_path_env.append_borrowed_to(&mut build_env);
        common_vars_env.append_borrowed_to(&mut build_env);

        build_env
    };
    pretty_print::step("Building contract");
    let mut wasm_artifact = cargo_native::compile::run::<Wasm>(
        &crate_metadata.manifest_path,
        &cargo_args,
        build_env,
        false,
        color,
    )?;

    wasm_artifact.path = {
        let prev_artifact_path = wasm_artifact.path;
        let target_path = output_paths.wasm_file;

        // target file does not yet exist `!target_path.is_file()` condition is implied by
        // `is_newer_than(...)` predicate, but it's redundantly added here for readability 🙏
        if !target_path.is_file() || is_newer_than(&prev_artifact_path, &target_path) {
            let (from_path, _maybe_tmpfile) =
                maybe_wasm_opt_step(&prev_artifact_path, args.no_wasmopt)?;
            crate::fs::copy_to_file(&from_path, &target_path)?;
        } else {
            println!();
            pretty_print::step(
                "Skipped running wasm-opt as final target exists and is newer than wasm produced by cargo",
            );
            println!();
        }
        target_path
    };

    wasm_artifact.builder_version_info = Some(builder_version_info);

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
        let pretty_abi_path = crate::fs::copy(&path, &output_paths.out_dir)?;
        messages.push_free(("ABI", pretty_abi_path.to_string().yellow().bold()));
    }
    if let Some(abi_path) = min_abi_path {
        messages.push_free(("Embedded ABI", abi_path.to_string().yellow().bold()));
    }

    messages.pretty_print();
    pretty_print::duration(start, "cargo near build");
    Ok(wasm_artifact)
}

fn is_newer_than(prev: &Utf8PathBuf, next: &Utf8PathBuf) -> bool {
    // (1) if `next` does not yet exist, `metadata_of_prev.modified()` will be greater than
    // `std::time::SystemTime::UNIX_EPOCH`;
    // (2) if `m.modified()` isn't available on current platform, the predicate will always
    // return true
    // (3) non-monotonic nature of `std::time::SystemTime` won't be a problem:
    // if the next_time and prev_time are too close in time so that next_time registers
    // before prev_time, it will only affect that skipping build won't occur, but doesn't
    // affect correctness, as the build will run next time due to prev_time > next_time
    let prev_time = std::fs::metadata(prev)
        .and_then(|m| m.modified())
        .unwrap_or_else(|_| std::time::SystemTime::now());
    let next_time = std::fs::metadata(next)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let debug_msg = format!(
        "{prev:?} = {prev_time:?}\n\
        {next:?} = {next_time:?}"
    );
    println!();
    println!(
        "Modification timestamps of:\n{}",
        pretty_print::indent_payload(&debug_msg)
    );
    prev_time > next_time
}

fn maybe_wasm_opt_step(
    input_path: &Utf8PathBuf,
    no_wasmopt: bool,
) -> eyre::Result<(Utf8PathBuf, Option<NamedTempFile>)> {
    let result = if !no_wasmopt {
        let opt_destination = tempfile::Builder::new()
            .prefix("optimized-")
            .suffix(".wasm")
            .tempfile()?;
        println!();
        pretty_print::handle_step(
            "Running an optimize for size post-step with wasm-opt...",
            || {
                let start = std::time::Instant::now();
                tracing::debug!(
                    "{} -> {}",
                    format!("{}", input_path).cyan(),
                    format!("{}", opt_destination.path().to_string_lossy()).cyan()
                );
                wasm_opt::OptimizationOptions::new_optimize_for_size()
                    .run(input_path, opt_destination.path())?;
                pretty_print::duration_millis(start, "wasm-opt -O");
                Ok(())
            },
        )?;

        (
            Utf8PathBuf::try_from(opt_destination.path().to_path_buf())?,
            Some(opt_destination),
        )
    } else {
        (input_path.clone(), None)
    };
    Ok(result)
}
