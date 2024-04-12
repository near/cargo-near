use std::ops::Deref;

use crate::{types::manifest::CargoManifestPath, util};

mod build;
mod docker;
pub const INSIDE_DOCKER_ENV_KEY: &str = "NEAR_BUILD_ENVIRONMENT_REF";

#[derive(Debug, Default, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = BuildCommandlContext)]
pub struct BuildCommand {
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
    /// Coloring: auto, always, never
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_interactive_input)]
    pub color: Option<crate::common::ColorPreference>,
}

#[derive(Debug)]
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
        if Self::no_docker() {
            self::build::run(self)
        } else {
            self.docker_run(context)
        }
    }
    pub fn no_docker() -> bool {
        std::env::var(INSIDE_DOCKER_ENV_KEY).is_ok()
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
            no_release: scope.no_release,
            no_abi: scope.no_abi,
            no_embed_abi: scope.no_embed_abi,
            no_doc: scope.no_doc,
            out_dir: scope.out_dir.clone(),
            manifest_path: scope.manifest_path.clone(),
            color: scope.color.clone(),
        };
        args.run(BuildContext::Build)?;
        Ok(Self)
    }
}
