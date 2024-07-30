use std::ops::Deref;

use colored::{ColoredString, Colorize};

use crate::{
    types::manifest::CargoManifestPath,
    util::{self, CompilationArtifact},
};

pub(crate) mod build;
mod docker;

// ====================== NEP-330 1.2.0 - Build Details Extension ===========
pub const NEP330_BUILD_ENVIRONMENT_ENV_KEY: &str = "NEP330_BUILD_INFO_BUILD_ENVIRONMENT";
pub const NEP330_BUILD_COMMAND_ENV_KEY: &str = "NEP330_BUILD_INFO_BUILD_COMMAND";
pub const NEP330_CONTRACT_PATH_ENV_KEY: &str = "NEP330_BUILD_INFO_CONTRACT_PATH";
pub const NEP330_SOURCE_CODE_SNAPSHOT_ENV_KEY: &str = "NEP330_BUILD_INFO_SOURCE_CODE_SNAPSHOT";
// ====================== End section =======================================

// ====================== NEP-330 1.1.0 - Contract Metadata Extension ===========
pub const NEP330_LINK_ENV_KEY: &str = "NEP330_LINK";
pub const NEP330_VERSION_ENV_KEY: &str = "NEP330_VERSION";
// ====================== End section =======================================

pub const CARGO_NEAR_VERSION_ENV_KEY: &str = "CARGO_NEAR_VERSION";
pub const CARGO_NEAR_ABI_SCHEMA_VERSION_ENV_KEY: &str = "CARGO_NEAR_ABI_SCHEMA_VERSION";

pub const BUILD_RS_ABI_STEP_HINT_ENV_KEY: &str = "CARGO_NEAR_ABI_GENERATION";

pub const SERVER_DISABLE_INTERACTIVE: &str = "CARGO_NEAR_SERVER_BUILD_DISABLE_INTERACTIVE";

#[derive(Debug, Default, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = BuildCommandlContext)]
pub struct BuildCommand {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    #[interactive_clap(long)]
    pub no_locked: bool,
    /// Build contract on host system and without embedding SourceScan NEP-330 metadata
    #[interactive_clap(long)]
    no_docker: bool,
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
    /// Set compile-time feature flags.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub features: Option<String>,
    /// Disables default feature flags.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub no_default_features: bool,
    /// Coloring: auto, always, never
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_interactive_input)]
    pub color: Option<crate::common::ColorPreference>,
}

#[derive(Debug, Clone, Copy)]
pub enum BuildContext {
    Build,
    Deploy,
}
impl BuildCommand {
    pub fn contract_path(&self) -> color_eyre::eyre::Result<camino::Utf8PathBuf> {
        let contract_path: camino::Utf8PathBuf = if let Some(manifest_path) = &self.manifest_path {
            let manifest_path = CargoManifestPath::try_from(manifest_path.deref().clone())?;
            manifest_path.directory()?.to_path_buf()
        } else {
            camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?).map_err(|err| {
                color_eyre::eyre::eyre!("Failed to convert path {}", err.to_string_lossy())
            })?
        };
        Ok(contract_path)
    }
    pub fn run(self, context: BuildContext) -> color_eyre::eyre::Result<util::CompilationArtifact> {
        if self.no_docker() {
            self::build::run(self.into())
        } else {
            self.docker_run(context)
        }
    }
    pub fn no_docker(&self) -> bool {
        std::env::var(NEP330_BUILD_ENVIRONMENT_ENV_KEY).is_ok() || self.no_docker
    }
}

#[derive(Default)]
pub struct ArtifactMessages<'a> {
    messages: Vec<(&'a str, ColoredString)>,
}

impl<'a> ArtifactMessages<'a> {
    pub fn push_binary(
        &mut self,
        wasm_artifact: &CompilationArtifact,
    ) -> color_eyre::eyre::Result<()> {
        self.messages.push((
            "Binary",
            wasm_artifact.path.to_string().bright_yellow().bold(),
        ));
        let checksum = wasm_artifact.compute_hash()?;
        self.messages.push((
            "SHA-256 checksum hex ",
            checksum.to_hex_string().green().dimmed(),
        ));
        self.messages.push((
            "SHA-256 checksum bs58",
            checksum.to_base58_string().green().dimmed(),
        ));
        Ok(())
    }
    pub fn push_free(&mut self, msg: (&'a str, ColoredString)) {
        self.messages.push(msg);
    }
    pub fn pretty_print(self) {
        let max_width = self.messages.iter().map(|(h, _)| h.len()).max().unwrap();
        for (header, message) in self.messages {
            eprintln!("     - {:>width$}: {}", header, message, width = max_width);
        }
    }
}

impl From<CliBuildCommand> for BuildCommand {
    fn from(value: CliBuildCommand) -> Self {
        Self {
            no_locked: value.no_locked,
            no_docker: value.no_docker,
            no_release: value.no_release,
            no_abi: value.no_abi,
            no_embed_abi: value.no_embed_abi,
            no_doc: value.no_doc,
            features: value.features,
            no_default_features: value.no_default_features,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            color: value.color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuildCommandlContext;

impl BuildCommandlContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        scope: &<BuildCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let args = BuildCommand {
            no_locked: scope.no_locked,
            no_docker: scope.no_docker,
            no_release: scope.no_release,
            no_abi: scope.no_abi,
            no_embed_abi: scope.no_embed_abi,
            no_doc: scope.no_doc,
            out_dir: scope.out_dir.clone(),
            manifest_path: scope.manifest_path.clone(),
            features: scope.features.clone(),
            no_default_features: scope.no_default_features,
            color: scope.color.clone(),
        };
        args.run(BuildContext::Build)?;
        Ok(Self)
    }
}
