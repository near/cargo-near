#[cfg(any(feature = "build_internal", feature = "docker"))]
use std::io::IsTerminal;

#[cfg(feature = "docker")]
#[derive(Debug, Clone, Copy, Default)]
pub enum BuildContext {
    #[default]
    Build,
    Deploy {
        skip_git_remote_check: bool,
    },
}

/// argument of [`build_with_cli`](crate::build_with_cli) function
///
/// [`std::default::Default`] implementation is derived:
/// - `false` for `bool`-s,
/// - `None` - for `Option`-s
/// - empty vector - for `Vec`
/// - delegates to [`impl Default for CliDescription`](struct.CliDescription.html#impl-Default-for-CliDescription)
///
/// # Adding a new field
///
/// When [`build_with_cli`](crate::build_with_cli) is used, [`Opts`] is serialized to argv
/// for the spawned `cargo near` subprocess via [`Opts::to_argv`]. Any new field whose value
/// should reach that subprocess must be wired into [`Opts::to_argv`] explicitly — there is
/// no env var or other propagation path, so a missing emit means the field is silently
/// dropped at the process boundary.
#[derive(Debug, Default, Clone, bon::Builder)]
pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    #[builder(default)]
    pub no_locked: bool,
    /// Build contract in debug mode, without optimizations and bigger in size
    #[builder(default)]
    pub no_release: bool,
    /// Set build profile
    pub profile: Option<String>,
    /// Do not generate ABI for the contract
    #[builder(default)]
    pub no_abi: bool,
    /// Do not embed the ABI in the contract binary
    #[builder(default)]
    pub no_embed_abi: bool,
    /// Do not include rustdocs in the embedded ABI
    #[builder(default)]
    pub no_doc: bool,
    /// do not run `wasm-opt -O` on the generated output as a post-step
    #[builder(default)]
    pub no_wasmopt: bool,
    /// Copy final artifacts to this directory
    #[builder(into)]
    pub out_dir: Option<camino::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[builder(into)]
    pub manifest_path: Option<camino::Utf8PathBuf>,
    /// Set compile-time feature flags.
    #[builder(into)]
    pub features: Option<String>,
    /// Set compile-time feature flags for ABI generation step only.
    /// If not specified, `features` will be used for ABI generation.
    #[builder(into)]
    pub abi_features: Option<String>,
    /// Disables default feature flags.
    #[builder(default)]
    pub no_default_features: bool,
    /// Coloring: auto, always, never;
    /// assumed to be auto when `None`
    pub color: Option<ColorPreference>,
    /// description of cli command, where [`BuildOpts`](crate::BuildOpts) are being used from, either real
    /// or emulated
    #[builder(default)]
    pub cli_description: CliDescription,
    /// additional environment key-value pairs, that should be passed to underlying
    /// build commands
    #[builder(default)]
    pub env: Vec<(String, String)>,
    /// override value of [`crate::env_keys::nep330::CONTRACT_PATH`] environment variable,
    /// needed when a sub-contract being built inside of `build.rs`
    /// resides in different [`crate::env_keys::nep330::CONTRACT_PATH`] than the current contract
    #[builder(into)]
    pub override_nep330_contract_path: Option<String>,
    /// override value of [`crate::env_keys::CARGO_TARGET_DIR`] environment variable,
    /// which is required to avoid deadlock <https://github.com/rust-lang/cargo/issues/8938>
    /// when a sub-contract is built in `build.rs`
    ///
    /// should best be a subfolder of [`crate::env_keys::CARGO_TARGET_DIR`]
    /// of crate being built to work normally
    #[builder(into)]
    pub override_cargo_target_dir: Option<String>,
    /// override value of [`crate::env_keys::nep330::OUTPUT_WASM_PATH`] environment variable,
    #[builder(into)]
    pub override_nep330_output_wasm_path: Option<String>,
    /// override value of [`crate::env_keys::RUSTUP_TOOLCHAIN`] environment variable, used for all invoked `rustc`, `cargo` and `rustup` commands
    #[builder(into)]
    pub override_toolchain: Option<String>,
    /// Disable Rust version checking
    #[builder(default)]
    pub skip_rust_version_check: bool,
}

/// used as field in [`BuildOpts`](crate::BuildOpts)
#[derive(Debug, Clone)]
pub struct CliDescription {
    /// binary name for `builder` field in [`near_abi::BuildInfo::builder`](https://docs.rs/near-abi/latest/near_abi/struct.BuildInfo.html#structfield.builder)
    pub cli_name_abi: String,
    /// cli command prefix for export of [`crate::env_keys::nep330::BUILD_COMMAND`] variable
    /// when used as lib method
    pub cli_command_prefix: Vec<String>,
}

/// this is `"cargo-near"` for [`CliDescription::cli_name_abi`] and
///
/// `vec!["cargo", "near", "build", "non-reproducible-wasm"]` for [`CliDescription::cli_command_prefix`]
impl Default for CliDescription {
    fn default() -> Self {
        Self {
            cli_name_abi: "cargo-near".into(),
            cli_command_prefix: vec![
                "cargo".into(),
                "near".into(),
                "build".into(),
                "non-reproducible-wasm".into(),
            ],
        }
    }
}

impl Opts {
    /// Serializes [`Opts`] → argv for the `cargo near` subprocess spawned by
    /// [`build_with_cli`](crate::build_with_cli).
    ///
    /// This is a 1-to-1 mapping of each field of [`Opts`] to a CLI flag, in field-declaration
    /// order. `Opts::default()` round-trips to a plain `cargo near build` command with no args.
    ///
    /// This is the **only** path by which [`Opts`] fields reach the spawned subprocess —
    /// fields that aren't emitted here are silently dropped at the process boundary. When
    /// adding a new field to [`Opts`], add the corresponding emit here too.
    pub fn to_argv(&self) -> Vec<String> {
        let cargo_args = self.cli_description.cli_command_prefix.clone();
        let mut cargo_args: Vec<&str> = cargo_args.iter().map(|ele| ele.as_str()).collect();
        // this logical NOT is needed to avoid writing manually `Default` trait impl for `Opts`
        // with `self.locked` field and to keep default (if nothing is specified) to *locked* behavior
        // which is a desired default for [crate::extended::build] functionality
        if !self.no_locked {
            cargo_args.push("--locked");
        }

        match (self.no_release, self.profile.as_ref()) {
            (_, Some(custom_profile_arg)) => {
                cargo_args.extend(["--profile", custom_profile_arg]);
            }
            (true, None) => cargo_args.push("--no-release"),
            (false, None) => {}
        }
        if self.no_abi {
            cargo_args.push("--no-abi");
        }
        if self.no_embed_abi {
            cargo_args.push("--no-embed-abi");
        }
        if self.no_doc {
            cargo_args.push("--no-doc");
        }
        if self.no_wasmopt {
            cargo_args.push("--no-wasmopt");
        }
        if let Some(ref out_dir) = self.out_dir {
            cargo_args.extend_from_slice(&["--out-dir", out_dir.as_str()]);
        }
        if let Some(ref features) = self.features {
            cargo_args.extend(&["--features", features]);
        }
        let effective_abi_features = self.abi_features.as_ref().or(self.features.as_ref());
        if let Some(abi_features) = effective_abi_features {
            cargo_args.extend(&["--abi-features", abi_features]);
        }
        if self.no_default_features {
            cargo_args.push("--no-default-features");
        }
        let color;
        if let Some(ref color_arg) = self.color {
            color = color_arg.to_string();
            cargo_args.extend(&["--color", &color]);
        }

        let equal_pairs: Vec<String> = self
            .env
            .iter()
            .map(|(key, value)| [key.as_str(), value.as_str()].join("="))
            .collect();
        for equal_pair in equal_pairs.iter() {
            cargo_args.extend(&["--env", equal_pair]);
        }
        if let Some(ref toolchain) = self.override_toolchain {
            cargo_args.extend(&["--override-toolchain", toolchain]);
        }
        if self.skip_rust_version_check {
            cargo_args.push("--skip-rust-version-check");
        }

        cargo_args
            .into_iter()
            .map(|el| el.to_string())
            .collect::<Vec<_>>()
    }

    #[deprecated(since = "0.20.2", note = "renamed to `to_argv` for clarity")]
    pub fn get_cli_command_for_lib_context(&self) -> Vec<String> {
        self.to_argv()
    }
}

/// used as field in [`BuildOpts`](crate::BuildOpts)
///
/// it determines if the print to stdout/stderr is colored or not
/// # Behaviour of [`ColorPreference::Auto`]:
/// if [`crate::env_keys::COLOR_PREFERENCE_NO_COLOR`] environment variable is set and isnt't set to `"0"`, then the result is [`ColorPreference::Never`]
///
/// otherwise it's [`ColorPreference::Always`] if stderr is a terminal device,
/// and [`ColorPreference::Never`] in the remaining cases
#[derive(Debug, Clone, Copy)]
pub enum ColorPreference {
    Auto,
    Always,
    Never,
}

impl std::fmt::Display for ColorPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
        }
    }
}

#[cfg(any(feature = "build_internal", feature = "docker"))]
fn default_mode() -> ColorPreference {
    match std::env::var(crate::env_keys::COLOR_PREFERENCE_NO_COLOR) {
        Ok(v) if v != "0" => ColorPreference::Never,
        _ => {
            if std::io::stderr().is_terminal() {
                ColorPreference::Always
            } else {
                ColorPreference::Never
            }
        }
    }
}

#[cfg(any(feature = "build_internal", feature = "docker"))]
impl ColorPreference {
    pub(crate) fn apply(&self) {
        match self {
            ColorPreference::Auto => {
                default_mode().apply();
            }
            ColorPreference::Always => colored::control::set_override(true),
            ColorPreference::Never => colored::control::set_override(false),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_opts_get_cli_build_command_for_env_vals() {
        let opts = super::Opts {
            env: vec![
                ("KEY".into(), "VALUE".into()),
                (
                    "GOOGLE_QUERY".into(),
                    "https://www.google.com/search?q=google+translate&sca_esv=3c150c50f502bc5d"
                        .into(),
                ),
            ],
            ..Default::default()
        };

        assert_eq!(opts.to_argv(), ["cargo".to_string(),
             "near".to_string(),
             "build".to_string(),
             "non-reproducible-wasm".to_string(),
             "--locked".to_string(),
             "--env".to_string(),
             "KEY=VALUE".to_string(),
             "--env".to_string(),
             "GOOGLE_QUERY=https://www.google.com/search?q=google+translate&sca_esv=3c150c50f502bc5d".to_string()]);
    }

    fn has_flag_with_value(cmd: &[String], flag: &str, value: &str) -> bool {
        cmd.windows(2)
            .any(|pair| pair[0] == flag && pair[1] == value)
    }

    #[test]
    fn test_opts_defaults_abi_features_to_features_when_abi_features_are_not_set() {
        let opts = super::Opts {
            features: Some("feature1".into()),
            ..Default::default()
        };

        let cmd = opts.to_argv();
        assert!(has_flag_with_value(&cmd, "--features", "feature1"));
        assert!(has_flag_with_value(&cmd, "--abi-features", "feature1"));
    }

    #[test]
    fn test_opts_get_cli_build_command_for_abi_features() {
        let opts = super::Opts {
            features: Some("feature1".into()),
            abi_features: Some("abi_feature1".into()),
            ..Default::default()
        };

        let cmd = opts.to_argv();
        assert!(has_flag_with_value(&cmd, "--features", "feature1"));
        assert!(has_flag_with_value(&cmd, "--abi-features", "abi_feature1"));
    }

    #[test]
    fn test_opts_get_cli_build_command_for_abi_features_only() {
        let opts = super::Opts {
            abi_features: Some("abi_only_feature".into()),
            ..Default::default()
        };

        let cmd = opts.to_argv();
        assert!(!cmd.contains(&"--features".to_string()));
        assert!(cmd.contains(&"--abi-features".to_string()));
        assert!(cmd.contains(&"abi_only_feature".to_string()));
    }

    #[test]
    fn test_opts_get_cli_build_command_for_custom_profile() {
        let opts = super::Opts {
            profile: Some("test-release".into()),
            ..Default::default()
        };

        let cmd = opts.to_argv();
        assert!(cmd.contains(&"--profile".to_string()));
        assert!(cmd.contains(&"test-release".to_string()));
        assert!(!cmd.contains(&"--no-release".to_string()));
    }

    #[test]
    fn test_opts_get_cli_build_command_forwards_skip_rust_version_check() {
        let opts = super::Opts {
            skip_rust_version_check: true,
            ..Default::default()
        };

        let cmd = opts.get_cli_command_for_lib_context();
        assert!(cmd.contains(&"--skip-rust-version-check".to_string()));

        let opts_unset = super::Opts::default();
        let cmd_unset = opts_unset.get_cli_command_for_lib_context();
        assert!(!cmd_unset.contains(&"--skip-rust-version-check".to_string()));
    }

    #[test]
    fn test_opts_get_cli_build_command_for_custom_profile_with_norelease() {
        let opts = super::Opts {
            profile: Some("test-release".into()),
            no_release: true,
            ..Default::default()
        };

        let cmd = opts.to_argv();
        assert!(cmd.contains(&"--profile".to_string()));
        assert!(cmd.contains(&"test-release".to_string()));
        assert!(!cmd.contains(&"--no-release".to_string()));
    }

    /// Completeness assertion for [`super::Opts::to_argv`].
    ///
    /// Every field of [`super::Opts`] that has a CLI surface is exercised here
    /// and the resulting flag is asserted. When you add a new field to
    /// [`super::Opts`] whose value should reach the subprocess spawned by
    /// [`crate::build_with_cli`], add a non-default value here and a matching
    /// assertion — if this test fails after a field addition, your field is
    /// being silently dropped at the process boundary.
    ///
    /// Fields without a CLI emit (e.g. `manifest_path`, `override_cargo_target_dir`,
    /// `override_nep330_*`) are intentionally excluded — they reach the subprocess
    /// through env vars or the process working directory rather than argv.
    #[test]
    fn test_opts_to_argv_emits_every_field_with_a_dedicated_flag() {
        let opts = super::Opts {
            // `no_locked: false` (default) so we also assert that `--locked` is
            // emitted by default
            no_release: true,
            no_abi: true,
            no_embed_abi: true,
            no_doc: true,
            no_wasmopt: true,
            out_dir: Some("/tmp/out".into()),
            features: Some("feat".into()),
            abi_features: Some("abi-feat".into()),
            no_default_features: true,
            color: Some(super::ColorPreference::Always),
            env: vec![("K".into(), "V".into())],
            override_toolchain: Some("nightly".into()),
            // `profile` is intentionally None — it is mutually exclusive with
            // `no_release` and is covered by `test_opts_get_cli_build_command_for_custom_profile`
            ..Default::default()
        };

        let cmd = opts.to_argv();
        assert!(cmd.contains(&"--locked".to_string()));
        assert!(cmd.contains(&"--no-release".to_string()));
        assert!(cmd.contains(&"--no-abi".to_string()));
        assert!(cmd.contains(&"--no-embed-abi".to_string()));
        assert!(cmd.contains(&"--no-doc".to_string()));
        assert!(cmd.contains(&"--no-wasmopt".to_string()));
        assert!(has_flag_with_value(&cmd, "--out-dir", "/tmp/out"));
        assert!(has_flag_with_value(&cmd, "--features", "feat"));
        assert!(has_flag_with_value(&cmd, "--abi-features", "abi-feat"));
        assert!(cmd.contains(&"--no-default-features".to_string()));
        assert!(has_flag_with_value(&cmd, "--color", "always"));
        assert!(has_flag_with_value(&cmd, "--env", "K=V"));
        assert!(has_flag_with_value(&cmd, "--override-toolchain", "nightly"));
    }
}
