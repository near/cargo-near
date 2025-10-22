use cargo_near_build::abi::AbiOpts;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = AbiCommandlContext)]
pub struct Command {
    /// Enable `--locked` flag for all `cargo` commands, disabled by default
    ///
    /// Running with `--locked` will fail, if
    /// 1. the contract's crate doesn't have a Cargo.lock file,
    ///    which locks in place the versions of all of the contract's dependencies
    ///    (and, recursively, dependencies of dependencies ...), or
    /// 2. if it has Cargo.lock file, but it needs to be updated (happens if Cargo.toml manifest was updated)
    ///    This just passes `--locked` to all downstream `cargo` commands being called.
    #[interactive_clap(long)]
    #[interactive_clap(verbatim_doc_comment)]
    pub locked: bool,
    /// Do not include rustdocs in the ABI file
    ///
    /// Specifying this flag results in not including human-readable documentation strings
    /// over contract's methods parsed from source code into ABI.
    /// More info about near ABI can be found here: [near/ABI](https://github.com/near/abi).
    #[interactive_clap(verbatim_doc_comment)]
    #[interactive_clap(long)]
    pub no_doc: bool,
    /// Generate compact (minified) JSON, no prettyprint, no whitespace
    #[interactive_clap(long)]
    pub compact_abi: bool,
    /// Copy final artifacts (`ABI.json`) to this directory
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` manifest of the contract crate to build
    ///
    /// If this argument is not specified, by default the `Cargo.toml` in current directory is assumed
    /// as the manifest of target crate to build.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    #[interactive_clap(verbatim_doc_comment)]
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Space or comma separated list of features to activate
    ///
    /// e.g. --features 'feature0 crate3/feature1 feature3'
    /// This just passes the argument as `--features` argument to downstream `cargo` command.
    /// Unlike `cargo` argument, this argument doesn't support repetition, at most 1 argument can be specified.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    #[interactive_clap(verbatim_doc_comment)]
    pub features: Option<String>,
    /// Whether to color output to stdout and stderr by printing ANSI escape sequences: auto, always, never
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
            features: value.features,
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
            features: scope.features.clone(),
            color: scope.color.clone(),
        };
        cargo_near_build::abi::build(args.into())?;
        Ok(Self)
    }
}
