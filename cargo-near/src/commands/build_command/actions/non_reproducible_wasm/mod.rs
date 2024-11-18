#[derive(Default, Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = cargo_near_build::BuildContext)]
#[interactive_clap(output_context = context::Context)]
pub struct BuildOpts {
    /// enable implicit `--locked` flag for all `cargo` commands, disabled by default
    #[interactive_clap(long)]
    pub locked: bool,
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
    /// Do not run `wasm-opt -O` on the generated output as a post-step
    #[interactive_clap(long)]
    pub no_wasmopt: bool,
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
    pub color: Option<crate::types::color_preference_cli::ColorPreferenceCli>,
    /// env overrides in the form of `"KEY=VALUE"` strings
    #[interactive_clap(long_vec_multiple_opt)]
    pub env: Vec<String>,
}

impl From<CliBuildOpts> for BuildOpts {
    fn from(value: CliBuildOpts) -> Self {
        Self {
            locked: value.locked,
            no_release: value.no_release,
            no_abi: value.no_abi,
            no_embed_abi: value.no_embed_abi,
            no_doc: value.no_doc,
            no_wasmopt: value.no_wasmopt,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            features: value.features,
            no_default_features: value.no_default_features,
            color: value.color,
            env: value.env,
            
        }
    }
}

pub mod context {
    
    #[derive(Debug)]
    pub struct Context;

    impl Context {
        pub fn from_previous_context(
            _previous_context: cargo_near_build::BuildContext,
            scope: &<super::BuildOpts as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
        ) -> color_eyre::eyre::Result<Self> {
            let opts = super::BuildOpts {
                locked: scope.locked,
                no_release: scope.no_release,
                no_abi: scope.no_abi,
                no_embed_abi: scope.no_embed_abi,
                no_doc: scope.no_doc,
                no_wasmopt: scope.no_wasmopt,
                features: scope.features.clone(),
                no_default_features: scope.no_default_features,
                env: scope.env.clone(),
                out_dir: scope.out_dir.clone(),
                manifest_path: scope.manifest_path.clone(),
                color: scope.color.clone(),
            };
            super::run(opts)?;
            Ok(Self)
        }
    }
}

pub fn run(cmd: BuildOpts) -> color_eyre::eyre::Result<String> {
    // if let BuildContext::Deploy {
    //     skip_git_remote_check: true,
    // } = context
    // {
    //     return Err(color_eyre::eyre::eyre!(
    //         "`--skip-git-remote-check` flag is only applicable for docker builds"
    //     ));
    // }
    println!("run_no_docker: {:#?}", cmd);
    Ok("no_docker artifact path".to_owned())
}
