use cargo_near_build::BuildContext;


#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = cargo_near_build::BuildContext)]
#[interactive_clap(output_context = OptsContext)]
pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    #[interactive_clap(long)]
    pub no_locked: bool,
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

#[derive(Debug)]
pub struct OptsContext; 

impl OptsContext {
    pub fn from_previous_context(
        previous_context: cargo_near_build::BuildContext,
        scope: &<Opts as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let opts = Opts {
            no_locked: scope.no_locked,
            out_dir: scope.out_dir.clone(),
            manifest_path: scope.manifest_path.clone(),
            color: scope.color.clone(),
        };
        run_docker(opts, previous_context)?;
        Ok(Self)
    }
}

pub fn run_docker(
    cmd: Opts,
    context: BuildContext,
) -> color_eyre::eyre::Result<()> {
    println!("run_docker: {:#?}, context {:?}", cmd, context);
    Ok(())
}
