use std::collections::HashMap;

use cargo_near_build::docker;
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

impl BuildCommand {
    fn validate_env_opt(&self) -> color_eyre::eyre::Result<()> {
        for pair in self.env.iter() {
            pair.split_once('=').ok_or(color_eyre::eyre::eyre!(
                "invalid \"key=value\" environment argument (must contain '='): {}",
                pair
            ))?;
        }
        Ok(())
    }
    pub fn run(self, context: BuildContext) -> color_eyre::eyre::Result<BuildArtifact> {
        self.validate_env_opt()?;
        if self.no_docker() {
            if let BuildContext::Deploy {
                skip_git_remote_check: true,
            } = context
            {
                return Err(color_eyre::eyre::eyre!(
                    "`--skip-git-remote-check` flag is only applicable for docker builds"
                ));
            }
            cargo_near_build::build(self.into())
        } else {
            let docker_opts = docker_opts_from((self, context));
            cargo_near_build::docker::build(docker_opts)
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
            no_wasmopt: value.no_wasmopt,
            features: value.features,
            no_default_features: value.no_default_features,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            color: value.color,
            env: value.env,
        }
    }
}

fn get_env_key_vals(input: Vec<String>) -> Vec<(String, String)> {
    let iterator = input.iter().flat_map(|pair_string| {
        pair_string
            .split_once('=')
            .map(|(env_key, value)| (env_key.to_string(), value.to_string()))
    });

    let dedup_map: HashMap<String, String> = HashMap::from_iter(iterator);

    let result = dedup_map.into_iter().collect();
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Passed additional environment pairs:\n{}",
        near_cli_rs::common::indent_payload(&format!("{:#?}", result))
    );
    result
}

impl From<BuildCommand> for BuildOpts {
    fn from(value: BuildCommand) -> Self {
        Self {
            no_locked: value.no_locked,
            no_release: value.no_release,
            no_abi: value.no_abi,
            no_embed_abi: value.no_embed_abi,
            no_doc: value.no_doc,
            no_wasmopt: value.no_wasmopt,
            features: value.features,
            no_default_features: value.no_default_features,
            out_dir: value.out_dir.map(Into::into),
            manifest_path: value.manifest_path.map(Into::into),
            color: value.color.map(Into::into),
            cli_description: Default::default(),
            env: get_env_key_vals(value.env),
            override_nep330_contract_path: None,
            override_cargo_target_dir: None,
        }
    }
}

/// this is more or less equivalent to
/// impl From<(BuildCommand, BuildContext)> for docker::DockerBuildOpts
/// which is not possible due to BuildContext being a non-local type to current (cli) crate
fn docker_opts_from(value: (BuildCommand, BuildContext)) -> docker::DockerBuildOpts {
    docker::DockerBuildOpts {
        no_locked: value.0.no_locked,
        no_release: value.0.no_release,
        no_abi: value.0.no_abi,
        no_embed_abi: value.0.no_embed_abi,
        no_doc: value.0.no_doc,
        no_wasmopt: value.0.no_wasmopt,
        features: value.0.features,
        no_default_features: value.0.no_default_features,
        out_dir: value.0.out_dir.map(Into::into),
        manifest_path: value.0.manifest_path.map(Into::into),
        color: value.0.color.map(Into::into),
        cli_description: Default::default(),
        env: get_env_key_vals(value.0.env),
        context: value.1,
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
            no_wasmopt: scope.no_wasmopt,
            out_dir: scope.out_dir.clone(),
            manifest_path: scope.manifest_path.clone(),
            features: scope.features.clone(),
            no_default_features: scope.no_default_features,
            color: scope.color.clone(),
            env: scope.env.clone(),
        };
        args.run(BuildContext::Build)?;
        Ok(Self)
    }
}
