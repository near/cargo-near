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
use crate::{ColorPreference, cargo_native, env_keys};
use crate::{
    cargo_native::target::COMPILATION_TARGET,
    pretty_print,
    types::{cargo::metadata::CrateMetadata, near::build::output::version_info::VersionInfo},
};

use super::abi;

/// Protocol version at which the nearcore VM accepts the bulk-memory +
/// nontrapping-float-to-int wasm opcodes that rustc >= 1.87 emits.
const BULK_MEMORY_PROTOCOL_VERSION: u32 = 84;

/// Max rustc version allowed for a contract whose `near-sdk` declares
/// `[package.metadata.near] min_protocol_version`. `None` means no ceiling (PV-84+, the
/// nearcore 2.12 VM accepts rustc 1.87+ opcodes); otherwise the historical 1.86 ceiling
/// applies, since rustc 1.87+ output is rejected by the pre-2.12 VM.
fn max_allowed_rustc(min_pv: Option<u32>) -> Option<rustc_version::Version> {
    let pv = min_pv.unwrap_or(0);
    if pv >= BULK_MEMORY_PROTOCOL_VERSION {
        None
    } else {
        Some(rustc_version::Version::new(1, 86, 0))
    }
}

fn checking_unsupported_toolchain(
    rustc_version: &rustc_version::Version,
    near_sdk_min_pv: Option<u32>,
) -> eyre::Result<()> {
    let Some(max_allowed) = max_allowed_rustc(near_sdk_min_pv) else {
        // No ceiling (PV-84+). Only note it when this rustc would have been rejected
        // under the historical 1.86 ceiling; otherwise stay quiet.
        if *rustc_version >= MIN_RUSTC_EMITTING_BULK_MEMORY_OPCODES {
            // `near_sdk_min_pv` is necessarily `Some(pv >= 84)` here: the only input
            // for which `max_allowed_rustc` returns `None`.
            let pv = near_sdk_min_pv.unwrap_or(BULK_MEMORY_PROTOCOL_VERSION);
            println!(
                "{}: {}",
                "INFO".green(),
                format!(
                    "contract's near-sdk targets protocol version {pv}; rustc {rustc_version} \
                    accepted (bulk-memory opcodes supported by nearcore VM)"
                )
                .cyan(),
            );
        }
        return Ok(());
    };
    // Reaching here means a ceiling applies (PV < 84 or absent metadata), so the
    // "upgrade near-sdk to declare min_protocol_version = 84" remediation below is valid.
    if *rustc_version > max_allowed {
        let pv_explanation = match near_sdk_min_pv {
            Some(pv) => format!("your contract's near-sdk targets protocol version {pv}"),
            None => "your contract's near-sdk targets protocol < 84 (no \
                `package.metadata.near.min_protocol_version` declared)"
                .to_string(),
        };
        println!(
            "{}: {} {} ({})",
            "WARNING".red(),
            "max rustc allowed:".yellow(),
            max_allowed.to_string().cyan(),
            pv_explanation.yellow(),
        );
        println!(
            "{}: {} {} {}",
            "WARNING".red(),
            "wasm, compiled with".yellow(),
            rustc_version.to_string().cyan(),
            "is not compatible with the nearcore VM at the protocol version your contract targets"
                .yellow(),
        );
        let downgrade_step =
            format!("cd /path/to/your/contract/project\nrustup override set {max_allowed}");
        let info_str = format!(
            "Step 1 - Set the Specific Rust Version for Your Project:\n{}\nStep 2 - Install the wasm32-unknown-unknown Target:\n{}",
            pretty_print::indent_payload(&downgrade_step),
            pretty_print::indent_payload("rustup target add wasm32-unknown-unknown")
        );
        println!(
            "{}: {} {} {}\n{}\n{} {}",
            "WARNING".red(),
            "please downgrade to".yellow(),
            max_allowed.to_string().cyan(),
            "toolchain for compiling contracts:".yellow(),
            pretty_print::indent_payload(&info_str),
            "OR".yellow(),
            "upgrade near-sdk to a release declaring `min_protocol_version = 84` (e.g. the nearcore-2.12 release)"
                .yellow(),
        );

        eyre::bail!(
            "wasm, compiled with rustc {rustc_version} exceeds the max allowed {max_allowed} for this contract"
        );
    }
    Ok(())
}

/// builds a contract whose crate root is current workdir, or identified by [`Cargo.toml`/BuildOpts::manifest_path](crate::BuildOpts::manifest_path) location
pub fn run(args: Opts) -> eyre::Result<CompilationArtifact> {
    let start = std::time::Instant::now();

    // Detect the effective toolchain to use: explicit override or active toolchain from rustup
    // This ensures consistent toolchain usage across version checking and actual build.
    // We detect the toolchain from the project directory (if manifest_path is provided)
    // to properly respect rust-toolchain.toml files in the target project.
    let project_dir = get_project_dir(args.manifest_path.as_ref());
    let effective_toolchain = args
        .override_toolchain
        .clone()
        .or_else(|| detect_active_toolchain(project_dir));

    let rustc_version = version_meta_with_override(effective_toolchain.clone())?.semver;

    let override_cargo_target_path_env =
        common_buildtime_env::CargoTargetDir::new(args.override_cargo_target_dir.clone());

    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    // Collected before the rustc version check so we can read `near-sdk`'s
    // `min_protocol_version` and pick the correct max-rustc threshold.
    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        CrateMetadata::get_with_build_opts(&args, &override_cargo_target_path_env)
    })?;

    if !args.skip_rust_version_check {
        pretty_print::handle_step("Checking rustc version...", || {
            let near_sdk_min_pv = crate_metadata.near_sdk_min_protocol_version();
            checking_unsupported_toolchain(&rustc_version, near_sdk_min_pv)
        })?;
    } else {
        pretty_print::step(
            &"WARN: Skipping rustc version check...\n"
                .yellow()
                .to_string(),
        );
    }

    pretty_print::handle_step("Checking the host environment...", || {
        if !cargo_native::target::wasm32_exists(effective_toolchain.clone()) {
            eyre::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
        }
        Ok(())
    })?;

    // addition of this check wasn't a change in logic, as previously output path was
    // assumed without `--out-dir` too, so docker-build was just failing if the arg was supplied:
    // https://github.com/near/cargo-near/blob/075d7b6dc9ab1f5c199edb6931512ccaf5af848e/cargo-near-build/src/types/near/docker_build/cloned_repo.rs#L100
    if env_keys::is_inside_docker_context() && args.out_dir.is_some() {
        return Err(eyre::eyre!(
            "inside docker build `--out-dir` is forbidden to be used in order to predict build output path in a straightforward way"
        ));
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

    // Resolve effective wasm-build rustflags as a token list, in priority order:
    //   1. default tokens: ["-C", "link-arg=-s"]
    //   2. user-provided RUSTFLAGS via args.env, parsed as whitespace-split tokens
    //   3. user-provided CARGO_ENCODED_RUSTFLAGS via args.env, parsed as 0x1f-split tokens
    //      (ENCODED wins over RUSTFLAGS, matching cargo's own precedence)
    // Then ["--cfg", "near"] is force-appended so it can't be dropped by user overrides —
    // it's semantically required to select the on-chain host-function path in near-sdk >= 5.27.
    //
    // We carry the result as CARGO_ENCODED_RUSTFLAGS (0x1f-separated) for robustness against
    // args containing spaces (e.g. paths in `-L`). Cargo prefers ENCODED over RUSTFLAGS when
    // both are set, so we forward neither RUSTFLAGS nor CARGO_ENCODED_RUSTFLAGS from args.env
    // afterward to avoid double-setting.
    let user_encoded =
        args.env.iter().rev().find_map(|(k, v)| {
            (k.as_str() == env_keys::CARGO_ENCODED_RUSTFLAGS).then_some(v.as_str())
        });
    let user_rustflags = args
        .env
        .iter()
        .rev()
        .find_map(|(k, v)| (k.as_str() == env_keys::RUSTFLAGS).then_some(v.as_str()));

    let mut rustflag_tokens: Vec<String> = if let Some(encoded) = user_encoded {
        encoded
            .split('\x1f')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect()
    } else if let Some(rustflags) = user_rustflags {
        rustflags.split_whitespace().map(String::from).collect()
    } else {
        vec!["-C".into(), "link-arg=-s".into()]
    };
    rustflag_tokens.push("--cfg".into());
    rustflag_tokens.push("near".into());
    let encoded_rustflags = rustflag_tokens.join("\x1f");

    let build_env = {
        let mut build_env: Vec<(&str, &str)> = vec![(
            env_keys::CARGO_ENCODED_RUSTFLAGS,
            encoded_rustflags.as_str(),
        )];
        // Forward all other args.env entries, but skip the rustflags carriers — they've already
        // been folded into `encoded_rustflags` above.
        build_env.extend(
            args.env
                .iter()
                .filter(|(k, _)| {
                    k.as_str() != env_keys::RUSTFLAGS
                        && k.as_str() != env_keys::CARGO_ENCODED_RUSTFLAGS
                })
                .map(|(key, value)| (key.as_str(), value.as_str())),
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
        let target_path = output_paths.get_wasm_file();

        // target file does not yet exist `!target_path.is_file()` condition is implied by
        // `is_newer_than(...)` predicate, but it's redundantly added here for readability 🙏
        if !target_path.is_file() || is_newer_than(&prev_artifact_path, target_path) {
            let (from_path, _maybe_tmpfile) =
                maybe_wasm_opt_step(&prev_artifact_path, args.no_wasmopt, &rustc_version)?;
            crate::fs::copy_to_file(&from_path, target_path)?;
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

/// Threshold at which rustc starts emitting wasm with bulk-memory + nontrapping-float-to-int
/// opcodes. At or above this version, `wasm-opt` must be told to enable those features so
/// it doesn't reject the input.
const MIN_RUSTC_EMITTING_BULK_MEMORY_OPCODES: rustc_version::Version =
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
            if *rustc_version >= MIN_RUSTC_EMITTING_BULK_MEMORY_OPCODES {
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

/// Detects the active toolchain that rustup would use for a given directory,
/// respecting directory overrides (rust-toolchain.toml, rustup override set).
/// Returns None if rustup is not available or fails to detect the toolchain.
///
/// # Arguments
/// * `project_dir` - Optional path to run rustup from. If None, uses current directory.
///
/// This function intentionally returns None rather than an error when rustup is unavailable,
/// allowing cargo-near to work in environments without rustup by falling back to the default
/// rustc behavior.
fn detect_active_toolchain(project_dir: Option<&camino::Utf8Path>) -> Option<String> {
    let mut cmd = std::process::Command::new("rustup");
    cmd.args(["show", "active-toolchain"]);

    if let Some(dir) = project_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().ok()?;

    if !output.status.success() {
        tracing::debug!("Failed to detect active toolchain: rustup command failed");
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    // The output format is: "toolchain-name (reason)"
    // e.g., "1.86.0-aarch64-apple-darwin (directory override for '/path/to/project')"
    // We extract just the toolchain name before the first space.
    // This parsing relies on rustup's stable output format for `show active-toolchain`.
    // Note: split_whitespace() already handles leading/trailing whitespace.
    stdout.split_whitespace().next().map(String::from)
}

/// Gets the project directory from the manifest path option.
/// If manifest_path is provided, returns its parent directory.
/// Otherwise returns None (will use current directory).
fn get_project_dir(manifest_path: Option<&camino::Utf8PathBuf>) -> Option<&camino::Utf8Path> {
    manifest_path.and_then(|p| p.parent())
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
    fn test_get_project_dir() {
        // Test with None - should return None
        assert!(get_project_dir(None).is_none());

        // Test with a valid path - should return parent
        let path = camino::Utf8PathBuf::from("/some/path/to/Cargo.toml");
        let result = get_project_dir(Some(&path));
        assert_eq!(result, Some(camino::Utf8Path::new("/some/path/to")));

        // Test with root path
        let root_path = camino::Utf8PathBuf::from("/Cargo.toml");
        let result = get_project_dir(Some(&root_path));
        assert_eq!(result, Some(camino::Utf8Path::new("/")));
    }

    #[test]
    fn test_max_allowed_rustc_back_compat_default() {
        // No metadata declared / older SDKs => historical 1.86 floor.
        assert_eq!(
            max_allowed_rustc(None),
            Some(rustc_version::Version::new(1, 86, 0))
        );
        // Below the PV-84 threshold => same floor.
        assert_eq!(
            max_allowed_rustc(Some(83)),
            Some(rustc_version::Version::new(1, 86, 0))
        );
        assert_eq!(
            max_allowed_rustc(Some(0)),
            Some(rustc_version::Version::new(1, 86, 0))
        );
    }

    #[test]
    fn test_max_allowed_rustc_pv84_lifts_ceiling() {
        // PV >= 84 => no ceiling at all.
        assert_eq!(max_allowed_rustc(Some(84)), None);
        assert_eq!(max_allowed_rustc(Some(99)), None);
    }

    #[test]
    fn test_checking_unsupported_toolchain_accepts_pinned_186() {
        // Pre-PV-84 SDK + rustc 1.86 => OK.
        let v186 = rustc_version::Version::new(1, 86, 0);
        assert!(checking_unsupported_toolchain(&v186, None).is_ok());
        assert!(checking_unsupported_toolchain(&v186, Some(83)).is_ok());
    }

    #[test]
    fn test_checking_unsupported_toolchain_rejects_193_without_metadata() {
        // 1.93 with no PV declared (back-compat default) must still fail.
        let v193 = rustc_version::Version::new(1, 93, 0);
        let err = checking_unsupported_toolchain(&v193, None).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("exceeds the max allowed"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn test_checking_unsupported_toolchain_accepts_193_with_pv84_metadata() {
        // 1.93 with PV >= 84 declared => OK.
        let v193 = rustc_version::Version::new(1, 93, 0);
        assert!(checking_unsupported_toolchain(&v193, Some(84)).is_ok());
    }

    #[test]
    fn test_checking_unsupported_toolchain_pv84_has_no_ceiling() {
        // Once the contract targets PV-84+, there is no upper bound on rustc.
        let v1931 = rustc_version::Version::new(1, 93, 1);
        assert!(checking_unsupported_toolchain(&v1931, Some(84)).is_ok());

        let v199 = rustc_version::Version::new(1, 99, 0);
        assert!(checking_unsupported_toolchain(&v199, Some(84)).is_ok());
        assert!(checking_unsupported_toolchain(&v199, Some(100)).is_ok());
    }

    #[test]
    fn test_checking_unsupported_toolchain_rejects_post_186_without_pv84() {
        // Historical ceiling preserved for PV < 84 / absent metadata: rustc beyond
        // 1.86 must still fail.
        let v1931 = rustc_version::Version::new(1, 93, 1);
        assert!(checking_unsupported_toolchain(&v1931, None).is_err());
        assert!(checking_unsupported_toolchain(&v1931, Some(83)).is_err());

        let v199 = rustc_version::Version::new(1, 99, 0);
        let err = checking_unsupported_toolchain(&v199, Some(83)).unwrap_err();
        assert!(format!("{err}").contains("exceeds the max allowed"));
    }

    #[test]
    fn test_detect_active_toolchain_respects_directory() {
        // This test verifies that detect_active_toolchain can detect toolchain
        // from a specific directory. We use the cargo-near workspace root which
        // has a rust-toolchain.toml.

        // Get the cargo-near workspace root (parent of cargo-near-build)
        let manifest_dir: camino::Utf8PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let workspace_root = manifest_dir
            .parent()
            .expect("cargo-near-build should have parent");

        // Detect toolchain from workspace root
        let result = detect_active_toolchain(Some(workspace_root));

        // If rustup is available, we should detect a toolchain
        if let Some(toolchain) = result {
            assert!(
                !toolchain.is_empty(),
                "Detected toolchain should not be empty"
            );
            // The workspace uses "stable" channel per rust-toolchain.toml
            // The detected toolchain should contain "stable" or be a specific version
            assert!(
                toolchain.contains("stable") || toolchain.chars().next().unwrap().is_ascii_digit(),
                "Toolchain should be 'stable' or a version number, got: {}",
                toolchain
            );
        }
    }
}
