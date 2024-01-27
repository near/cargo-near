pub mod build;

#[derive(Debug, Default, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = BuildCommandlContext)]
pub struct BuildCommand {
    /// Build contract in release mode, with optimizations
    #[interactive_clap(long)]
    pub release: bool,
    /// Embed the ABI in the contract binary
    #[interactive_clap(long)]
    pub embed_abi: bool,
    /// Include rustdocs in the embedded ABI
    #[interactive_clap(long)]
    pub doc: bool,
    /// Do not generate ABI for the contract
    #[interactive_clap(long, conflicts_with_all = &["doc", "embed_abi"])]
    pub no_abi: bool,
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

#[derive(Debug, Clone)]
pub struct BuildCommandlContext;

impl BuildCommandlContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        scope: &<BuildCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let args = BuildCommand {
            release: scope.release,
            embed_abi: scope.embed_abi,
            doc: scope.doc,
            no_abi: scope.no_abi,
            out_dir: scope.out_dir.clone(),
            manifest_path: scope.manifest_path.clone(),
            color: scope.color.clone(),
        };
        self::build::run(args)?;
        Ok(Self)
    }
}
