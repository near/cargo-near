use cargo_near_build::BuildContext;

use cargo_near_build::docker;
use cargo_near_build::BuildArtifact;

#[derive(Debug, Default, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = cargo_near_build::BuildContext)]
#[interactive_clap(output_context = context::Context)]
pub struct BuildOpts {
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

impl From<CliBuildOpts> for BuildOpts {
    fn from(value: CliBuildOpts) -> Self {
        Self {
            no_locked: value.no_locked,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            color: value.color,
        }
    }
}

mod context {
    #[derive(Debug)]
    pub struct Context;

    impl Context {
        pub fn from_previous_context(
            previous_context: cargo_near_build::BuildContext,
            scope: &<super::BuildOpts as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
        ) -> color_eyre::eyre::Result<Self> {
            let opts = super::BuildOpts {
                no_locked: scope.no_locked,
                out_dir: scope.out_dir.clone(),
                manifest_path: scope.manifest_path.clone(),
                color: scope.color.clone(),
            };
            super::run(opts, previous_context)?;
            Ok(Self)
        }
    }
}

/// this is more or less equivalent to
/// impl From<(BuildCommand, BuildContext)> for docker::DockerBuildOpts
/// which is not possible due to BuildContext being a non-local type to current (cli) crate
fn docker_opts_from(value: (BuildOpts, BuildContext)) -> docker::DockerBuildOpts {
    docker::DockerBuildOpts {
        no_locked: value.0.no_locked,
        out_dir: value.0.out_dir.map(Into::into),
        manifest_path: value.0.manifest_path.map(Into::into),
        color: value.0.color.map(Into::into),
        context: value.1,
    }
}

pub fn run(opts: BuildOpts, context: BuildContext) -> color_eyre::eyre::Result<BuildArtifact> {
    let docker_opts = docker_opts_from((opts, context));
    cargo_near_build::docker::build(docker_opts)
}
