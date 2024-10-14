use crate::types::cargo::manifest_path::ManifestPath;
use crate::{CliDescription, ColorPreference};

use super::build::input::BuildContext;

pub mod cloned_repo;
pub mod crate_in_repo;
pub mod metadata;

mod compute_command;
pub mod subprocess;

#[derive(Default, Debug, Clone, bon::Builder)]
pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    #[builder(default)]
    pub no_locked: bool,
    /// Build contract in debug mode, without optimizations and bigger in size
    #[builder(default)]
    pub no_release: bool,
    /// Do not generate ABI for the contract
    #[builder(default)]
    pub no_abi: bool,
    /// Do not embed the ABI in the contract binary
    #[builder(default)]
    pub no_embed_abi: bool,
    /// Do not include rustdocs in the embedded ABI
    #[builder(default)]
    pub no_doc: bool,
    /// Copy final artifacts to this directory
    pub out_dir: Option<camino::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    pub manifest_path: Option<camino::Utf8PathBuf>,
    /// Set compile-time feature flags.
    #[builder(into)]
    pub features: Option<String>,
    /// Disables default feature flags.
    #[builder(default)]
    pub no_default_features: bool,
    /// Coloring: auto, always, never;
    /// assumed to be auto when `None`
    pub color: Option<ColorPreference>,
    /// description of cli command, where [BuildOpts](crate::BuildOpts) are being used from, either real
    /// or emulated
    #[builder(default)]
    pub cli_description: CliDescription,
    /// additional environment key-value pairs, that should be passed to underlying
    /// build commands
    #[builder(default)]
    pub env: Vec<(String, String)>,
    #[builder(default)]
    pub context: BuildContext,
}

impl Default for BuildContext {
    fn default() -> Self {
        Self::Build
    }
}

pub const WARN_BECOMES_ERR: &str =
    "This WARNING becomes a hard ERROR when deploying contract with docker.";

impl Opts {
    pub fn contract_path(&self) -> eyre::Result<camino::Utf8PathBuf> {
        let contract_path: camino::Utf8PathBuf = if let Some(manifest_path) = &self.manifest_path {
            let manifest_path = ManifestPath::try_from(manifest_path.clone())?;
            manifest_path.directory()?.to_path_buf()
        } else {
            camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
                .map_err(|err| eyre::eyre!("Failed to convert path {}", err.to_string_lossy()))?
        };
        Ok(contract_path)
    }
}
