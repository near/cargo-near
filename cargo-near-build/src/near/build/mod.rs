use crate::cargo_native::Wasm;
use crate::types::near::abi as abi_types;
use crate::types::near::build::{buildtime_env, common_buildtime_env};
use camino::Utf8PathBuf;
use colored::Colorize;
use near_abi::BuildInfo;
use tempfile::NamedTempFile;

use crate::types::near::build::input::Opts;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::build::side_effects::ArtifactMessages;
use crate::{cargo_native, env_keys, ColorPreference};
use crate::{
    cargo_native::target::COMPILATION_TARGET,
    pretty_print,
    types::{cargo::metadata::CrateMetadata, near::build::output::version_info::VersionInfo},
};

use super::abi;

fn checking_unsupported_toolchain(rustc_version: &rustc_version::Version) -> eyre::Result<()> {
    if *rustc_version >= MIN_VERSION_WITH_BULK_MEMORY_NTRAPPING_FLOAT_TO_INT {
        println!(
            "{}: {} {} {}",
            "WARNING".red(),
            "wasm, compiled with".yellow(),
            MIN_VERSION_WITH_BULK_MEMORY_NTRAPPING_FLOAT_TO_INT
                .to_string()
                .cyan(),
            "or newer rust toolchain is currently not compatible with nearcore VM".yellow()
        );
        let info_str = format!(
            "Step 1 - Set the Specific Rust Version for Your Project:\n{}\nStep 2 - Install the wasm32-unknown-unknown Target:\n{}",
            pretty_print::indent_payload(
                "cd /path/to/your/contract/project\nrustup override set 1.86"
            ),
            pretty_print::indent_payload("rustup target add wasm32-unknown-unknown")
        );
        println!(
            "{}: {} {} {}\n{}",
            "WARNING".red(),
            "please downgrade to".yellow(),
            MAX_VERSION_NO_BULK_MEMORY.to_string().cyan(),
            "toolchain for compiling contracts:".yellow(),
            pretty_print::indent_payload(&info_str)
        );

        eyre::bail!("wasm, compiled with {MIN_VERSION_WITH_BULK_MEMORY_NTRAPPING_FLOAT_TO_INT} or newer rust toolchain is currently not compatible with nearcore VM");
    }
    Ok(())
}

/// builds a contract whose crate root is current workdir, or identified by [`Cargo.toml`/BuildOpts::manifest_path](crate::BuildOpts::manifest_path) location
pub fn run(args: Opts) -> eyre::Result<CompilationArtifact> {
    let start = std::time::Instant::now();

    // Detect the effective toolchain to use: explicit override or active toolchain from rustup
    // This ensures consistent toolchain usage across version checking and actual build
    let effective_toolchain = args.override_toolchain.clone().or_else(detect_active_toolchain);

    let rustc_version = version_meta_with_override(effective_toolchain.clone())?.semver;

    if !args.skip_rust_version_check {
        checking_unsupported_toolchain(&rustc_version)?;
    }

    let override_cargo_target_path_env =
        common_buildtime_env::CargoTargetDir::new(args.override_cargo_target_dir.clone());

    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    pretty_print::handle_step("Checking the host environment...", || {
        if !cargo_native::target::wasm32_exists(effective_toolchain.clone()) {
            eyre::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
        }
        Ok(())
    })?;

    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        CrateMetadata::get_with_build_opts(&args, &override_cargo_target_path_env)
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

    // Features for ABI generation - use abi_features if set, otherwise fall back to features
    let abi_feature_args = {
        let mut feat_args = vec![];
        let features_for_abi = args.abi_features.as_ref().or(args.features.as_ref());
        if let Some(features) = features_for_abi {
            feat_args.extend(&["--features", features.as_str()]);
        }
        if args.no_default_features {
            feat_args.push("--no-default-features");
        }
        feat_args
    };

    // Features for WASM build - always use regular features
    let wasm_feature_args = {
        let mut feat_args = vec![];
        if let Some(features) = args.features.as_ref() {
            feat_args.extend(&["--features", features.as_str()]);
        }
        if args.no_default_features {
            feat_args.push("--no-default-features");
        }
        feat_args
    };

    match (args.no_release, args.profile.as_ref()) {
        (_, Some(custom_profile_arg)) => {
            cargo_args.extend(["--profile", custom_profile_arg]);
        }
        (false, None) => cargo_args.extend(["--profile", "release"]),
        (true, None) => {}
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
    env_keys::print_nep330_env();

    if !args.no_abi {
        let mut contract_abi = {
            let mut abi_env = args
                .env
                .iter()
                .map(|(key, value)| (key.as_ref(), value.as_ref()))
                .collect::<Vec<_>>();
            common_vars_env.append_borrowed_to(&mut abi_env);

            effective_toolchain.as_ref().inspect(|toolchain| {
                abi_env.push((env_keys::RUSTUP_TOOLCHAIN, toolchain));
            });

            abi::generate::procedure(
                &crate_metadata,
                args.no_locked,
                !args.no_doc,
                true,
                &abi_feature_args,
                &abi_env,
                color,
            )?
        };

        let embedding_binary = args.cli_description.cli_name_abi;
        contract_abi.metadata.build = Some(BuildInfo {
            compiler: format!("rustc {rustc_version}"),
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
            min_abi_path.replace(crate::fs::copy(&path, output_paths.get_out_dir())?);
        }
        abi = Some(contract_abi);
    }

    cargo_args.extend(wasm_feature_args);

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

        effective_toolchain.as_ref().inspect(|toolchain| {
            build_env.push((env_keys::RUSTUP_TOOLCHAIN, toolchain));
        });

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
        let target_path = output_paths.get_wasm_file().clone();

        // target file does not yet exist `!target_path.is_file()` condition is implied by
        // `is_newer_than(...)` predicate, but it's redundantly added here for readability 🙏
        if !target_path.is_file() || is_newer_than(&prev_artifact_path, &target_path) {
            let (from_path, _maybe_tmpfile) =
                maybe_wasm_opt_step(&prev_artifact_path, args.no_wasmopt, &rustc_version)?;
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
        let pretty_abi_path = crate::fs::copy(&path, output_paths.get_out_dir())?;
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

const MAX_VERSION_NO_BULK_MEMORY: rustc_version::Version = rustc_version::Version::new(1, 86, 0);
const MIN_VERSION_WITH_BULK_MEMORY_NTRAPPING_FLOAT_TO_INT: rustc_version::Version =
    rustc_version::Version::new(1, 87, 0);
fn maybe_wasm_opt_step(
    input_path: &Utf8PathBuf,
    no_wasmopt: bool,
    rustc_version: &rustc_version::Version,
) -> eyre::Result<(Utf8PathBuf, Option<NamedTempFile>)> {
    let result = if !no_wasmopt {
        let opt_destination = tempfile::Builder::new()
            .prefix("optimized-")
            .suffix(".wasm")
            .tempfile()?;
        println!();
        let additional_features = {
            let mut features = vec![];
            if *rustc_version >= MIN_VERSION_WITH_BULK_MEMORY_NTRAPPING_FLOAT_TO_INT {
                features.push((
                    wasm_opt::Feature::TruncSat,
                    "--enable-nontrapping-float-to-int",
                ));
                features.push((wasm_opt::Feature::BulkMemory, "--enable-bulk-memory"));
            }
            features
        };
        let msgs = additional_features
            .iter()
            .map(|el| el.1)
            .collect::<Vec<_>>();
        pretty_print::handle_step(
            &format!(
                "Running an optimize for size post-step with wasm-opt {}...",
                msgs.join(" ")
            ),
            || {
                let start = std::time::Instant::now();
                tracing::debug!(
                    "{} -> {}",
                    format!("{input_path}").cyan(),
                    format!("{}", opt_destination.path().to_string_lossy()).cyan()
                );
                let optimization_opts = {
                    let mut opts = wasm_opt::OptimizationOptions::new_optimize_for_size();
                    for feature in additional_features {
                        opts.enable_feature(feature.0);
                    }

                    opts
                };
                optimization_opts.run(input_path, opt_destination.path())?;
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

/// Detects the active toolchain that rustup would use, respecting directory overrides.
/// Returns None if rustup is not available or fails to detect the toolchain.
/// 
/// This function intentionally returns None rather than an error when rustup is unavailable,
/// allowing cargo-near to work in environments without rustup by falling back to the default
/// rustc behavior.
fn detect_active_toolchain() -> Option<String> {
    let output = std::process::Command::new("rustup")
        .args(["show", "active-toolchain"])
        .output()
        .ok()?;

    if !output.status.success() {
        tracing::debug!("Failed to detect active toolchain: rustup command failed");
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    // The output format is: "toolchain-name (reason)"
    // e.g., "1.86.0-aarch64-apple-darwin (directory override for '/path/to/project')"
    // We extract just the toolchain name before the first space.
    // This parsing relies on rustup's stable output format for `show active-toolchain`.
    stdout.trim().split_whitespace().next().map(String::from)
}

pub fn version_meta_with_override(
    override_toolchain: Option<String>,
) -> rustc_version::Result<rustc_version::VersionMeta> {
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| std::ffi::OsString::from("rustc"));
    let mut cmd = if let Some(wrapper) = std::env::var_os("RUSTC_WRAPPER").filter(|w| !w.is_empty())
    {
        let mut cmd = std::process::Command::new(wrapper);
        cmd.arg(rustc);
        cmd
    } else {
        std::process::Command::new(rustc)
    };
    cmd.arg("-vV");
    
    if let Some(toolchain) = override_toolchain {
        cmd.env(env_keys::RUSTUP_TOOLCHAIN, toolchain);
    }
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Command execution:\n{}",
        pretty_print::indent_payload(&format!("{cmd:#?}"))
    );

    let out = cmd
        .output()
        .map_err(rustc_version::Error::CouldNotExecuteCommand)?;

    if !out.status.success() {
        return Err(rustc_version::Error::CommandError {
            stdout: String::from_utf8_lossy(&out.stdout).into(),
            stderr: String::from_utf8_lossy(&out.stderr).into(),
        });
    }

    rustc_version::version_meta_for(std::str::from_utf8(&out.stdout)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_active_toolchain_format() {
        // Test the parsing logic with sample rustup output
        let sample_output = "1.86.0-x86_64-unknown-linux-gnu (directory override for '/path/to/project')\n";
        let toolchain = sample_output.trim().split_whitespace().next();
        assert_eq!(toolchain, Some("1.86.0-x86_64-unknown-linux-gnu"));
        
        // Test with leading/trailing whitespace
        let sample_with_whitespace = "  1.86.0-aarch64-apple-darwin (default)  \n";
        let toolchain = sample_with_whitespace.trim().split_whitespace().next();
        assert_eq!(toolchain, Some("1.86.0-aarch64-apple-darwin"));
    }

    #[test]
    fn test_detect_active_toolchain() {
        // This test verifies that detect_active_toolchain works when rustup is available
        // It will return None if rustup is not available, which is acceptable
        let result = detect_active_toolchain();
        
        // If rustup is available and working, we should get a non-empty toolchain name
        if let Some(toolchain) = result {
            assert!(!toolchain.is_empty(), "Detected toolchain should not be empty");
            assert!(!toolchain.contains(char::is_whitespace), "Toolchain should not contain whitespace");
            // Toolchain names typically contain version numbers and target triple
            assert!(toolchain.contains('-') || toolchain.chars().any(|c| c.is_numeric()), 
                    "Toolchain should contain version info or target triple");
        }
    }
}
