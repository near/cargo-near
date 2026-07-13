use crate::types::near::build::buildtime_env::Nep330CrateVars;
use crate::types::near::build::common_buildtime_env;
use crate::types::near::build::output::version_info::VersionInfo;
use crate::types::near::check::Opts;
use crate::{ColorPreference, cargo_native, env_keys};
use crate::{
    cargo_native::target::COMPILATION_TARGET, pretty_print, types::cargo::metadata::CrateMetadata,
};

use super::build::{detect_active_toolchain, encoded_rustflags_with_cfg_near, get_project_dir};

/// Type-checks a contract under the exact same environment [`build::run`](crate::build) uses,
/// without producing a wasm artifact.
///
/// Runs `cargo check` (default) or `cargo clippy` (when [`Opts::clippy`] is set) with:
/// - `--cfg near` force-appended to `CARGO_ENCODED_RUSTFLAGS` (identical logic to build, so
///   near-sdk >= 5.27 selects the on-chain host-function path),
/// - target `wasm32-unknown-unknown` (verified installed first),
/// - the same `--features` / `--no-default-features` / `--profile` (`--release` by default
///   unless `no_release`) / `--locked` resolution as build,
/// - the active or overridden toolchain.
///
/// Deliberately skipped vs. build (all only matter when emitting wasm): ABI generation and
/// embedding, `wasm-opt`, output-path copying, and the rustc/protocol-version ceiling check.
///
/// cargo diagnostics are streamed through; a non-zero cargo exit is propagated as an `Err`.
pub fn run(args: Opts) -> eyre::Result<()> {
    let start = std::time::Instant::now();

    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    // Detect the effective toolchain to use: explicit override or active toolchain from rustup,
    // resolved from the project directory so rust-toolchain.toml is respected. Mirrors build.
    let project_dir = get_project_dir(args.manifest_path.as_ref());
    let effective_toolchain = args
        .override_toolchain
        .clone()
        .or_else(|| detect_active_toolchain(project_dir));

    let override_cargo_target_path_env = common_buildtime_env::CargoTargetDir::NoOp;

    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        let manifest_path =
            crate::types::cargo::manifest_path::ManifestPath::from_manifest_path_opt(
                args.manifest_path.clone(),
            )?;
        CrateMetadata::collect(
            manifest_path,
            args.no_locked,
            &override_cargo_target_path_env,
            effective_toolchain.clone(),
        )
    })?;

    pretty_print::handle_step("Checking the host environment...", || {
        if !cargo_native::target::wasm32_exists(effective_toolchain.clone()) {
            eyre::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
        }
        Ok(())
    })?;

    let mut cargo_args = vec!["--target", COMPILATION_TARGET];

    if let Some(features) = args.features.as_ref() {
        cargo_args.extend(&["--features", features.as_str()]);
    }
    if args.no_default_features {
        cargo_args.push("--no-default-features");
    }

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

    // Identical `--cfg near` rustflags assembly to build. Carried as CARGO_ENCODED_RUSTFLAGS,
    // so we must not also forward RUSTFLAGS/CARGO_ENCODED_RUSTFLAGS from args.env below.
    let encoded_rustflags = encoded_rustflags_with_cfg_near(&args.env);

    // Reproduce the same crate-identity NEP-330 build-time env `cargo near build` exposes (via
    // `CommonVariables`, which shares this exact `Nep330CrateVars` group), so a contract whose
    // source or `build.rs` reads these variables type-checks against the same configuration
    // `build` would compile. The artifact/output-tied vars (output wasm path, contract-path and
    // cargo-target-dir overrides) are intentionally omitted — a `check` produces no artifact.
    let crate_vars = Nep330CrateVars::new(
        &crate_metadata,
        &VersionInfo::get_coerced_builder_version()?,
        || vec!["cargo".to_string(), "near".to_string(), "check".to_string()],
    )?;

    let check_env = {
        let mut check_env: Vec<(&str, &str)> = vec![(
            env_keys::CARGO_ENCODED_RUSTFLAGS,
            encoded_rustflags.as_str(),
        )];
        check_env.extend(
            args.env
                .iter()
                .filter(|(k, _)| {
                    k.as_str() != env_keys::RUSTFLAGS
                        && k.as_str() != env_keys::CARGO_ENCODED_RUSTFLAGS
                })
                .map(|(key, value)| (key.as_str(), value.as_str())),
        );

        crate_vars.append_borrowed_to(&mut check_env);

        effective_toolchain.as_ref().inspect(|toolchain| {
            check_env.push((env_keys::RUSTUP_TOOLCHAIN, toolchain));
        });

        check_env
    };

    let kind = if args.clippy {
        crate::types::near::check::CheckKind::Clippy
    } else {
        crate::types::near::check::CheckKind::Check
    };
    let subcommand = kind.cargo_subcommand();

    pretty_print::step(&format!("Checking contract (cargo {subcommand})"));
    cargo_native::compile::run_check(
        subcommand,
        &crate_metadata.manifest_path,
        &cargo_args,
        check_env,
        color,
    )?;

    pretty_print::success(&format!(
        "Contract checked successfully! (cargo {subcommand})"
    ));
    pretty_print::duration(start, &format!("cargo near check (cargo {subcommand})"));
    Ok(())
}
