pub mod abi;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = AbiCommandlContext)]
pub struct AbiCommand {
    /// Include rustdocs in the ABI file
    #[interactive_clap(long)]
    pub no_doc: bool,
    /// add --locked to corresponding `cargo` commands
    #[interactive_clap(long)]
    pub locked: bool,
    /// Generate compact (minified) JSON
    #[interactive_clap(long)]
    pub compact_abi: bool,
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
pub struct AbiCommandlContext;

impl AbiCommandlContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        scope: &<AbiCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let args = AbiCommand {
            no_doc: scope.no_doc,
            locked: scope.locked,
            compact_abi: scope.compact_abi,
            out_dir: scope.out_dir.clone(),
            manifest_path: scope.manifest_path.clone(),
            color: scope.color.clone(),
        };
        self::abi::run(args)?;
        Ok(Self)
    }
}
