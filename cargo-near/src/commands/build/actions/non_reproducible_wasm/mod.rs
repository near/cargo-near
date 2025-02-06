use cargo_near_build::BuildArtifact;

#[derive(Default, Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = cargo_near_build::BuildContext)]
#[interactive_clap(output_context = context::Context)]
pub struct BuildOpts {
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
    /// Build contract in `dev` profile, without optimizations and MUCH bigger in size
    ///
    /// Without the flag `cargo-near` passes `--release` to downstream `cargo` command during wasm build step.
    /// When the flag is specified, `--relase` isn't passed to `cargo`, and default `dev` profile is used
    #[interactive_clap(verbatim_doc_comment)]
    #[interactive_clap(long)]
    pub no_release: bool,
    /// Do not generate ABI for the contract, which is generated by default
    ///
    /// More info about near ABI can be found here: [near/ABI](https://github.com/near/abi).
    /// Default build runs in a few steps:
    /// 1. runs the equivalent of `cargo near abi`, generating a `*.json` file.
    /// 2. compresses `*.json` obtained in step (1.) into `*.zst` file.
    /// 3. runs `cargo build --target wasm32-unknown-unknown` and embeds `*.zst` file from step (2.)   
    ///    into contract's result `*.wasm` file at the same time.
    ///
    /// Specifying the flag results in only step (3.) being run without embedding any `*.zst` with ABI data.
    #[interactive_clap(verbatim_doc_comment)]
    #[interactive_clap(long)]
    pub no_abi: bool,
    /// Generates `*.json` ABI file but does NOT embed it into the contract's result `*.wasm` binary
    ///
    /// More info about near ABI can be found here: [near/ABI](https://github.com/near/abi).
    #[interactive_clap(verbatim_doc_comment)]
    #[interactive_clap(long)]
    pub no_embed_abi: bool,
    /// Do not include rustdocs in the embedded ABI
    ///
    /// Specifying this flag results in not including human-readable documentation strings
    /// over contract's methods parsed from source code into ABI.
    /// More info about near ABI can be found here: [near/ABI](https://github.com/near/abi).
    #[interactive_clap(verbatim_doc_comment)]
    #[interactive_clap(long)]
    pub no_doc: bool,
    /// Do not run equivalent of `wasm-opt -O` on the generated output as a post-step
    ///
    /// This `wasm-opt -O` step is required for correctness of build on rustc >= 1.82.
    /// Otherwise, if the post-step is skipped, nearcore runtime won't be able to parse and validate deployed wasm.
    /// In these cases a `CompilationError(PrepareError(Deserialization))` is expected to be emitted
    /// when performing calls to wasm with unsupported features.
    #[interactive_clap(long)]
    #[interactive_clap(verbatim_doc_comment)]
    pub no_wasmopt: bool,
    /// Copy final artifacts (`contract.wasm`, `ABI.json`, `ABI.zst`) to this directory
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
    /// Do not activate the `default` feature of contract's crate
    ///
    /// This just passes `--no-default-features` argument to downstream `cargo` command.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    #[interactive_clap(verbatim_doc_comment)]
    pub no_default_features: bool,
    /// Whether to color output to stdout and stderr by printing ANSI escape sequences: auto, always, never
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_interactive_input)]
    pub color: Option<crate::types::color_preference_cli::ColorPreferenceCli>,
    /// Environment overrides in the form of `"KEY=VALUE"` strings. This flag can be repeated.
    ///
    /// This is generally not needed to be used explicitly during regular development.
    /// This makes sense to be used to specify custom `RUSTFLAGS` for wasm build:
    /// ```bash
    /// cargo near build non-reproducible-wasm --env 'RUSTFLAGS=--verbose'
    /// RUST_LOG=info cargo near build non-reproducible-wasm --env 'RUSTFLAGS=--verbose -C link-arg=-s'
    /// ```
    /// This is also used with `passed_env` config in Cargo.toml manifest during reproducible builds,
    /// as an internal detail, allowing to persist info about environment in contract's metadata.
    #[interactive_clap(verbatim_doc_comment)]
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

impl From<BuildOpts> for cargo_near_build::BuildOpts {
    fn from(value: BuildOpts) -> Self {
        Self {
            no_locked: !value.locked,
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
            env: env_pairs::get_key_vals(value.env),
            override_nep330_contract_path: None,
            override_cargo_target_dir: None,
        }
    }
}

mod env_pairs {
    use std::collections::HashMap;

    impl super::BuildOpts {
        pub(super) fn validate_env_opt(&self) -> color_eyre::eyre::Result<()> {
            for pair in self.env.iter() {
                pair.split_once('=').ok_or(color_eyre::eyre::eyre!(
                    "invalid \"key=value\" environment argument (must contain '='): {}",
                    pair
                ))?;
            }
            Ok(())
        }
    }

    pub(super) fn get_key_vals(input: Vec<String>) -> Vec<(String, String)> {
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
}

pub mod rule {
    use color_eyre::Section;
    use colored::Colorize;

    const COMMAND_ERR_MSG: &str = "`container_build_command` is required to start with";

    fn is_inside_docker_context() -> bool {
        std::env::var(cargo_near_build::env_keys::nep330::BUILD_ENVIRONMENT).is_ok()
    }
    pub fn assert_locked(opts: &super::BuildOpts) {
        if is_inside_docker_context() {
            assert!(
                opts.locked,
                "build command should have `--locked` flag in docker"
            );
        }
    }

    fn get_docker_image() -> String {
        std::env::var(cargo_near_build::env_keys::nep330::BUILD_ENVIRONMENT).unwrap_or_else(|_| {
            panic!(
                "`{}` is expected to be set",
                cargo_near_build::env_keys::nep330::BUILD_ENVIRONMENT
            )
        })
    }
    pub fn enforce_this_program_args() -> color_eyre::eyre::Result<()> {
        if is_inside_docker_context() {
            let args = std::env::args().collect::<Vec<_>>();
            let default_cmd =
                cargo_near_build::BuildOpts::default().get_cli_command_for_lib_context();
            let default_cmd_len = default_cmd.len();
            if (args.len() < default_cmd_len)
                || (args[1..default_cmd_len] != default_cmd[1..default_cmd_len])
            {
                return Err(color_eyre::eyre::eyre!(
                    "{}\n`{}` for the used image:\n{}",
                    COMMAND_ERR_MSG,
                    serde_json::to_string(&default_cmd).unwrap(),
                    get_docker_image()
                )
                .note(format!(
                    "The default `{}` has changed since `{}` image\n\
                    See {}",
                    "container_build_command".cyan(),
                    "sourcescan/cargo-near:0.13.0-rust-1.83.0".cyan(),
                    "https://github.com/near/cargo-near/releases".cyan()
                )));
            }
        }
        Ok(())
    }
}

pub fn run(opts: BuildOpts) -> color_eyre::eyre::Result<BuildArtifact> {
    rule::assert_locked(&opts);
    opts.validate_env_opt()?;
    cargo_near_build::build(opts.into())
}
