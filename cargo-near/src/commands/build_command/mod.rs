use cargo_near_build::{env_keys, BuildArtifact, BuildOpts};
use colored::{ColoredString, Colorize};

pub(crate) mod build;
mod docker;

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

#[derive(Debug, Clone, Copy)]
pub enum BuildContext {
    Build,
    Deploy,
}
impl BuildCommand {
    pub fn run(self, context: BuildContext) -> color_eyre::eyre::Result<BuildArtifact> {
        if self.no_docker() {
            self::build::run(self.into())
        } else {
            let opts: docker::Opts = self.into();
            opts.docker_run(context)
        }
    }
    pub fn no_docker(&self) -> bool {
        std::env::var(env_keys::nep330::BUILD_ENVIRONMENT).is_ok() || self.no_docker
    }
}

#[derive(Default)]
pub struct ArtifactMessages<'a> {
    messages: Vec<(&'a str, ColoredString)>,
}

impl<'a> ArtifactMessages<'a> {
    pub fn push_binary(&mut self, artifact: &BuildArtifact) -> color_eyre::eyre::Result<()> {
        self.messages
            .push(("Binary", artifact.path.to_string().bright_yellow().bold()));
        let checksum = artifact.compute_hash()?;
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
