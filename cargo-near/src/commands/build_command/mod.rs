use cargo_near_build::{env_keys, BuildArtifact, BuildContext, BuildOpts};

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
    pub color: Option<crate::types::color_preference_cli::ColorPreferenceCli>,
}

impl BuildCommand {
    pub fn run(self, context: BuildContext) -> color_eyre::eyre::Result<BuildArtifact> {
        if self.no_docker() {
            match context {
                BuildContext::Deploy {
                    skip_git_remote_check,
                } if skip_git_remote_check == true => {
                    return Err(color_eyre::eyre::eyre!(
                        "`--skip-git-remote-check` flag is only applicable for docker builds"
                    ));
                }
                _ => {}
            }
            cargo_near_build::build(self.into())
        } else {
            cargo_near_build::docker::build(cargo_near_build::docker::DockerBuildOpts {
                build_opts: self.into(),
                context,
            })
        }
    }
    pub fn no_docker(&self) -> bool {
        std::env::var(env_keys::nep330::BUILD_ENVIRONMENT).is_ok() || self.no_docker
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

impl From<BuildCommand> for BuildOpts {
    fn from(value: BuildCommand) -> Self {
        Self {
            no_locked: value.no_locked,
            no_release: value.no_release,
            no_abi: value.no_abi,
            no_embed_abi: value.no_embed_abi,
            no_doc: value.no_doc,
            features: value.features,
            no_default_features: value.no_default_features,
            out_dir: value.out_dir.map(Into::into),
            manifest_path: value.manifest_path.map(Into::into),
            color: value.color.map(Into::into),
            cli_description: Default::default(),
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
