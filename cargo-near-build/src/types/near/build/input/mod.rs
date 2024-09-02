use std::env;
use std::io::IsTerminal;

#[cfg(feature = "docker")]
mod docker_context;

#[derive(Debug, Clone, Copy)]
pub enum BuildContext {
    Build,
    Deploy,
}

/// argument of [build](crate::build) function
///
/// [std::default::Default] implementation is derived:
/// - `false` for `bool`-s,
/// - `None` - for `Option`-s
/// - delegates to [impl Default for CliDescription](struct.CliDescription.html#impl-Default-for-CliDescription)
#[derive(Debug, Default, Clone)]
pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    pub no_locked: bool,
    /// Build contract in debug mode, without optimizations and bigger in size
    pub no_release: bool,
    /// Do not generate ABI for the contract
    pub no_abi: bool,
    /// Do not embed the ABI in the contract binary
    pub no_embed_abi: bool,
    /// Do not include rustdocs in the embedded ABI
    pub no_doc: bool,
    /// Copy final artifacts to this directory
    pub out_dir: Option<camino::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    pub manifest_path: Option<camino::Utf8PathBuf>,
    /// Set compile-time feature flags.
    pub features: Option<String>,
    /// Disables default feature flags.
    pub no_default_features: bool,
    /// Coloring: auto, always, never;
    /// assumed to be auto when `None`
    pub color: Option<ColorPreference>,
    /// description of cli command, where [BuildOpts](crate::BuildOpts) are being used from, either real
    /// or emulated
    pub cli_description: CliDescription,
}

/// used as field in [BuildOpts](crate::BuildOpts)
#[derive(Debug, Clone)]
pub struct CliDescription {
    /// binary name for `builder` field in [near_abi::BuildInfo::builder]
    pub cli_name_abi: String,
    /// cli command prefix for export of [crate::env_keys::nep330::BUILD_COMMAND] variable
    /// when used as lib method
    pub cli_command_prefix: Vec<String>,
}

/// this is `"cargo-near"` for [CliDescription::cli_name_abi] and
///
/// `vec!["cargo", "near", "build"]` for [CliDescription::cli_command_prefix]
impl Default for CliDescription {
    fn default() -> Self {
        Self {
            cli_name_abi: "cargo-near".into(),
            cli_command_prefix: vec!["cargo".into(), "near".into(), "build".into()],
        }
    }
}

impl Opts {
    /// this is just 1-to-1 mapping of each struct's field to a cli flag
    /// in order of fields, as specified in struct's definition.
    /// `Default` implementation corresponds to plain `cargo near build` command without any args
    pub(crate) fn get_cli_build_command(&self) -> Vec<String> {
        let cargo_args = self.cli_description.cli_command_prefix.clone();
        let mut cargo_args: Vec<&str> = cargo_args.iter().map(|ele| ele.as_str()).collect();
        if self.no_locked {
            cargo_args.push("--no-locked");
        }
        // `no_docker` field isn't present
        if self.no_release {
            cargo_args.push("--no-release");
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
        if let Some(ref out_dir) = self.out_dir {
            cargo_args.extend_from_slice(&["--out-dir", out_dir.as_str()]);
        }
        if let Some(ref manifest_path) = self.manifest_path {
            cargo_args.extend_from_slice(&["--manifest-path", manifest_path.as_str()]);
        }
        if let Some(ref features) = self.features {
            cargo_args.extend(&["--features", features]);
        }
        if self.no_default_features {
            cargo_args.push("--no-default-features");
        }
        let color;
        if let Some(ref color_arg) = self.color {
            color = color_arg.to_string();
            cargo_args.extend(&["--color", &color]);
        }
        cargo_args
            .into_iter()
            .map(|el| el.to_string())
            .collect::<Vec<_>>()
    }
}

/// used as field in [BuildOpts](crate::BuildOpts)
///
/// it determines if the print to stdout/stderr is colored or not
/// # Behaviour of [ColorPreference::Auto]:
/// if `NO_COLOR` environment variable is set and isnt't set to `"0"`, then the result is [ColorPreference::Never]
///
/// otherwise it's [ColorPreference::Always] if stderr is a terminal device,
/// and [ColorPreference::Never] in the remaining cases
#[derive(Debug, Clone)]
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

fn default_mode() -> ColorPreference {
    match env::var("NO_COLOR") {
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
