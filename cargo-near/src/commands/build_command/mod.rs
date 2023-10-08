pub mod build;

#[derive(Debug, Default, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[interactive_clap(skip_default_from_cli)]
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
    #[interactive_clap(skip_default_input_arg)]
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBufInner>,
    /// Path to the `Cargo.toml` of the contract to build
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBufInner>,
    /// Coloring: auto, always, never
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    pub color: Option<crate::common::ColorPreference>,
}

impl interactive_clap::FromCli for BuildCommand {
    type FromCliContext = near_cli_rs::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        _context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let clap_variant = optional_clap_variant.unwrap_or_default();
        let args = Self {
            release: clap_variant.release,
            embed_abi: clap_variant.embed_abi,
            doc: clap_variant.doc,
            no_abi: clap_variant.no_abi,
            out_dir: clap_variant.out_dir.clone(),
            manifest_path: clap_variant.manifest_path.clone(),
            color: clap_variant.color.clone(),
        };
        if let Err(err) = self::build::run(args).map(|_| ()) {
            interactive_clap::ResultFromCli::Err(Some(clap_variant), color_eyre::eyre::eyre!(err))
        } else {
            interactive_clap::ResultFromCli::Ok(clap_variant)
        }
    }
}
