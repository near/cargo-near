use cargo_near_build::check::CheckOpts;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = CheckCommandlContext)]
pub struct Command {
    /// Run `cargo clippy` instead of `cargo check`
    ///
    /// By default `cargo near check` runs the equivalent of `cargo check`. With this flag it
    /// runs `cargo clippy` instead, surfacing clippy lints in addition to type errors.
    #[interactive_clap(long)]
    #[interactive_clap(verbatim_doc_comment)]
    pub clippy: bool,
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
    /// Check the contract in `dev` profile, without optimizations
    ///
    /// Without the flag `cargo-near` passes `--release` to the downstream `cargo` command.
    /// When the flag is specified, `--release` isn't passed and the default `dev` profile is used.
    #[interactive_clap(verbatim_doc_comment)]
    #[interactive_clap(long)]
    pub no_release: bool,
    /// Check the contract with a custom build profile.
    ///
    /// This just passes the argument as `--profile` argument to the downstream `cargo` command.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub profile: Option<String>,
    /// Path to the `Cargo.toml` manifest of the contract crate to check
    ///
    /// If this argument is not specified, by default the `Cargo.toml` in current directory is assumed
    /// as the manifest of target crate to check.
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
    /// This makes sense to be used to specify custom `RUSTFLAGS` for the check, e.g.:
    /// ```bash
    /// cargo near check --env 'RUSTFLAGS=--verbose'
    /// ```
    /// In all cases `--cfg near` is force-appended to the resulting RUSTFLAGS by cargo-near, so
    /// `near-sdk` selects the on-chain host-function path (identical to `cargo near build`).
    #[interactive_clap(verbatim_doc_comment)]
    #[interactive_clap(long_vec_multiple_opt)]
    pub env: Vec<String>,
    /// override value of `RUSTUP_TOOLCHAIN` environment variable, used for all invoked `rustc`, `cargo` and `rustup` commands
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub override_toolchain: Option<String>,
}

impl Command {
    fn validate_env_opt(&self) -> color_eyre::eyre::Result<()> {
        for pair in self.env.iter() {
            pair.split_once('=').ok_or(color_eyre::eyre::eyre!(
                "invalid \"key=value\" environment argument (must contain '='): {}",
                pair
            ))?;
        }
        Ok(())
    }
}

impl From<Command> for CheckOpts {
    fn from(value: Command) -> Self {
        Self {
            clippy: value.clippy,
            no_locked: !value.locked,
            no_release: value.no_release,
            profile: value.profile,
            manifest_path: value.manifest_path.map(Into::into),
            features: value.features,
            no_default_features: value.no_default_features,
            color: value.color.map(Into::into),
            env: get_key_vals(value.env),
            override_toolchain: value.override_toolchain,
        }
    }
}

fn get_key_vals(input: Vec<String>) -> Vec<(String, String)> {
    use std::collections::HashMap;

    let iterator = input.iter().flat_map(|pair_string| {
        pair_string
            .split_once('=')
            .map(|(env_key, value)| (env_key.to_string(), value.to_string()))
    });

    // Deduplicate repeated `KEY=VALUE` pairs (last value wins), matching `cargo near build`'s
    // `env_pairs::get_key_vals` so a repeated `--env KEY=...` behaves identically across commands.
    let dedup_map: HashMap<String, String> = HashMap::from_iter(iterator);

    let result = dedup_map.into_iter().collect();
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Passed additional environment pairs:\n{}",
        near_cli_rs::common::indent_payload(&format!("{result:#?}"))
    );
    result
}

#[derive(Debug, Clone)]
pub struct CheckCommandlContext;

impl CheckCommandlContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        scope: &<Command as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let args = Command {
            clippy: scope.clippy,
            locked: scope.locked,
            no_release: scope.no_release,
            profile: scope.profile.clone(),
            manifest_path: scope.manifest_path.clone(),
            features: scope.features.clone(),
            no_default_features: scope.no_default_features,
            color: scope.color.clone(),
            env: scope.env.clone(),
            override_toolchain: scope.override_toolchain.clone(),
        };
        args.validate_env_opt()?;
        cargo_near_build::check::check(args.into())?;
        Ok(Self)
    }
}
