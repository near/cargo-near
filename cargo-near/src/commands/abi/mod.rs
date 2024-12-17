use cargo_near_build::abi::AbiOpts;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = AbiCommandlContext)]
pub struct Command {
    /// enable `--locked` flag for all `cargo` commands, disabled by default
    #[interactive_clap(long)]
    pub locked: bool,
    /// Include rustdocs in the ABI file
    #[interactive_clap(long)]
    pub no_doc: bool,
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
    pub color: Option<crate::types::color_preference_cli::ColorPreferenceCli>,
}

impl From<Command> for AbiOpts {
    fn from(value: Command) -> Self {
        Self {
            no_locked: !value.locked,
            no_doc: value.no_doc,
            compact_abi: value.compact_abi,
            out_dir: value.out_dir.map(Into::into),
            manifest_path: value.manifest_path.map(Into::into),
            color: value.color.map(Into::into),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbiCommandlContext;

impl AbiCommandlContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        scope: &<Command as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let args = Command {
            locked: scope.locked,
            no_doc: scope.no_doc,
            compact_abi: scope.compact_abi,
            out_dir: scope.out_dir.clone(),
            manifest_path: scope.manifest_path.clone(),
            color: scope.color.clone(),
        };
        cargo_near_build::abi::build(args.into())?;
        Ok(Self)
    }
}
