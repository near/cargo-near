use camino::Utf8PathBuf;
use cargo_near_build::cargo_native;
use cargo_near_build::near::abi;
use cargo_near_build::pretty_print;
use cargo_near_build::types::cargo::manifest_path::{ManifestPath, MANIFEST_FILE_NAME};
use cargo_near_build::types::cargo::metadata::CrateMetadata;
use cargo_near_build::types::near::abi as abi_types;
use cargo_near_build::types::near::VersionMismatch;
use cargo_near_build::WASM;
use colored::Colorize;
use near_abi::BuildInfo;

use crate::commands::build_command::{
    NEP330_BUILD_COMMAND_ENV_KEY, NEP330_CONTRACT_PATH_ENV_KEY, NEP330_SOURCE_CODE_SNAPSHOT_ENV_KEY,
};
use crate::BuildArtifact;
use cargo_near_build::types::color_preference::ColorPreference;

use super::{
    ArtifactMessages, CARGO_NEAR_ABI_SCHEMA_VERSION_ENV_KEY, CARGO_NEAR_VERSION_ENV_KEY,
    NEP330_BUILD_ENVIRONMENT_ENV_KEY, NEP330_LINK_ENV_KEY, NEP330_VERSION_ENV_KEY,
};

const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

// These `Opts` have no `no_docker` flag, i.e., they are only indented for build
// inside of a specific environment
#[derive(Debug, Default, Clone)]
pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    pub no_locked: bool,
    /// Build contract in debug mode, without optimizations and bigger is size
    pub no_release: bool,
    /// Do not generate ABI for the contract
    pub no_abi: bool,
    /// Do not embed the ABI in the contract binary
    pub no_embed_abi: bool,
    /// Do not include rustdocs in the embedded ABI
    pub no_doc: bool,
    /// Copy final artifacts to this directory
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Set compile-time feature flags.
    pub features: Option<String>,
    /// Disables default feature flags.
    pub no_default_features: bool,
    /// Coloring: auto, always, never
    pub color: Option<cargo_near_build::types::color_preference::ColorPreference>,
}

impl Opts {
    /// this is just 1-to-1 mapping of each struct's field to a cli flag
    /// in order of fields, as specified in struct's definition.
    /// `Default` implementation corresponds to plain `cargo near build` command without any args
    fn get_cli_build_command(&self) -> Vec<String> {
        let mut cargo_args = vec!["cargo", "near", "build"];
        if self.no_locked {
            cargo_args.push("--no-locked");
        }
        // `no_docker` field isn't present
        if self.no_release {
            cargo_args.push("--no-release");
        }
        if self.no_abi {
            cargo_args.push("--no-abi");
        }
        if self.no_embed_abi {
            cargo_args.push("--no-embed-abi");
        }
        if self.no_doc {
            cargo_args.push("--no-doc");
        }
        if let Some(ref out_dir) = self.out_dir {
            cargo_args.extend_from_slice(&["--out-dir", out_dir.as_str()]);
        }
        if let Some(ref manifest_path) = self.manifest_path {
            cargo_args.extend_from_slice(&["--manifest-path", manifest_path.as_str()]);
        }
        if let Some(ref features) = self.features {
            cargo_args.extend(&["--features", features]);
        }
        if self.no_default_features {
            cargo_args.push("--no-default-features");
        }
        let color;
        if let Some(ref color_arg) = self.color {
            color = color_arg.to_string();
            cargo_args.extend(&["--color", &color]);
        }
        cargo_args
            .into_iter()
            .map(|el| el.to_string())
            .collect::<Vec<_>>()
    }
}

impl From<super::BuildCommand> for Opts {
    fn from(value: super::BuildCommand) -> Self {
        Self {
            no_locked: value.no_locked,
            no_release: value.no_release,
            no_abi: value.no_abi,
            no_embed_abi: value.no_embed_abi,
            no_doc: value.no_doc,
            features: value.features,
            no_default_features: value.no_default_features,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            color: value.color.map(Into::into),
        }
    }
}

pub fn run(args: Opts) -> color_eyre::eyre::Result<BuildArtifact> {
    export_cargo_near_abi_versions();
    export_nep_330_build_command(&args)?;
    print_nep_330_env();

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
    let (cargo_near_version, cargo_near_version_mismatch) = coerce_cargo_near_version()?;
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
    build_env.push((NEP330_VERSION_ENV_KEY, &version));
    // this will be set in docker builds (externally to current process), having more info about git commit
    if std::env::var(NEP330_LINK_ENV_KEY).is_err() {
        if let Some(ref repository) = crate_metadata.root_package.repository {
            build_env.push((NEP330_LINK_ENV_KEY, repository));
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
        std::env::var(NEP330_BUILD_ENVIRONMENT_ENV_KEY).unwrap_or("host".into())
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

fn export_cargo_near_abi_versions() {
    if std::env::var(CARGO_NEAR_VERSION_ENV_KEY).is_err() {
        std::env::set_var(CARGO_NEAR_VERSION_ENV_KEY, env!("CARGO_PKG_VERSION"));
    }
    if std::env::var(CARGO_NEAR_ABI_SCHEMA_VERSION_ENV_KEY).is_err() {
        std::env::set_var(
            CARGO_NEAR_ABI_SCHEMA_VERSION_ENV_KEY,
            near_abi::SCHEMA_VERSION,
        );
    }
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
        NEP330_BUILD_COMMAND_ENV_KEY,
        serde_json::to_string(&env_value)?,
    );
    Ok(())
}

fn print_nep_330_env() {
    log::info!("Variables, relevant for reproducible builds:");
    for key in [
        NEP330_BUILD_ENVIRONMENT_ENV_KEY,
        NEP330_BUILD_COMMAND_ENV_KEY,
        NEP330_CONTRACT_PATH_ENV_KEY,
        NEP330_SOURCE_CODE_SNAPSHOT_ENV_KEY,
    ] {
        let value = std::env::var(key)
            .map(|val| format!("'{}'", val))
            .unwrap_or("unset".to_string());
        log::info!("{}={}", key, value);
    }
}

// TODO: make this an associated method for `VersionMismatch` type
fn coerce_cargo_near_version() -> color_eyre::eyre::Result<(String, VersionMismatch)> {
    match std::env::var(CARGO_NEAR_ABI_SCHEMA_VERSION_ENV_KEY) {
        Ok(env_near_abi_schema_version) => {
            if env_near_abi_schema_version != near_abi::SCHEMA_VERSION {
                return Err(color_eyre::eyre::eyre!(
                    "current process NEAR_ABI_SCHEMA_VERSION mismatch with env value: {} vs {}",
                    near_abi::SCHEMA_VERSION,
                    env_near_abi_schema_version,
                ));
            }
        }
        Err(_err) => {}
    }
    let current_version = env!("CARGO_PKG_VERSION");

    let result = match std::env::var(CARGO_NEAR_VERSION_ENV_KEY) {
        Err(_err) => (current_version.to_string(), VersionMismatch::None),
        Ok(env_version) => match env_version == current_version {
            true => (current_version.to_string(), VersionMismatch::None),
            // coercing to env_version on mismatch
            false => (
                env_version.clone(),
                VersionMismatch::Some {
                    environment: env_version,
                    current_process: current_version.to_string(),
                },
            ),
        },
    };
    Ok(result)
}
