use std::{env, str::FromStr};

pub use near_cli_rs::CliResult;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod abi;
pub mod build;
mod cargo;
mod types;
pub mod util;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct NearArgs {
    #[interactive_clap(subcommand)]
    pub cmd: NearCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ())]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum NearCommand {
    #[strum_discriminants(strum(
        message = "build   -   Build a NEAR contract and optionally embed ABI"
    ))]
    /// Build a NEAR contract and optionally embed ABI
    Build(BuildCommand),
    #[strum_discriminants(strum(message = "abi     -   Generates ABI for the contract"))]
    /// Generates ABI for the contract
    Abi(AbiCommand),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ())]
#[interactive_clap(skip_default_from_cli)]
pub struct AbiCommand {
    /// Include rustdocs in the ABI file
    #[interactive_clap(long)]
    pub doc: bool,
    /// Generate compact (minified) JSON
    #[interactive_clap(long)]
    pub compact_abi: bool,
    /// Copy final artifacts to this directory
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Coloring: auto, always, never
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    pub color: Option<ColorPreference>,
}

impl interactive_clap::FromCli for AbiCommand {
    type FromCliContext = ();
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();
        let doc = clap_variant.doc;
        let compact_abi = clap_variant.compact_abi;
        if clap_variant.out_dir.is_none() {
            clap_variant.out_dir = match Self::input_out_dir(&context) {
                Ok(optional_out_dir) => optional_out_dir,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let out_dir = clap_variant.out_dir.clone();
        if clap_variant.manifest_path.is_none() {
            clap_variant.manifest_path = match Self::input_manifest_path(&context) {
                Ok(optional_manifest_path) => optional_manifest_path,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let manifest_path = clap_variant.manifest_path.clone();
        if clap_variant.color.is_none() {
            clap_variant.color = match Self::input_color(&context) {
                Ok(optional_color) => optional_color,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let color = clap_variant.color.clone();

        let args = Self {
            doc,
            compact_abi,
            out_dir,
            manifest_path,
            color,
        };
        if let Err(err) = abi::run(args) {
            interactive_clap::ResultFromCli::Err(Some(clap_variant), color_eyre::Report::msg(err))
        } else {
            interactive_clap::ResultFromCli::Ok(clap_variant)
        }
    }
}

impl AbiCommand {
    fn input_color(_context: &()) -> color_eyre::eyre::Result<Option<ColorPreference>> {
        Ok(None)
    }

    fn input_out_dir(
        _context: &(),
    ) -> color_eyre::eyre::Result<Option<crate::types::utf8_path_buf::Utf8PathBuf>> {
        Ok(None)
    }

    fn input_manifest_path(
        _context: &(),
    ) -> color_eyre::eyre::Result<Option<crate::types::utf8_path_buf::Utf8PathBuf>> {
        Ok(None)
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ())]
#[interactive_clap(skip_default_from_cli)]
pub struct BuildCommand {
    /// Build contract in release mode, with optimizations
    #[interactive_clap(long)]
    pub release: bool,
    /// Embed the ABI in the contract binary
    #[interactive_clap(long)]
    pub embed_abi: bool,
    /// Include rustdocs in the embedded ABI
    #[interactive_clap(long)]
    pub doc: bool,
    /// Do not generate ABI for the contract
    #[interactive_clap(long)]
    // #[clap(long, conflicts_with_all = &["doc", "embed-abi"])]
    pub no_abi: bool,
    /// Copy final artifacts to this directory
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Coloring: auto, always, never
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    pub color: Option<ColorPreference>,
}

impl interactive_clap::FromCli for BuildCommand {
    type FromCliContext = ();
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();
        let release = clap_variant.release;
        let embed_abi = clap_variant.embed_abi;
        let doc = clap_variant.doc;
        let no_abi = clap_variant.no_abi;
        if clap_variant.out_dir.is_none() {
            clap_variant.out_dir = match Self::input_out_dir(&context) {
                Ok(optional_out_dir) => optional_out_dir,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let out_dir = clap_variant.out_dir.clone();
        if clap_variant.manifest_path.is_none() {
            clap_variant.manifest_path = match Self::input_manifest_path(&context) {
                Ok(optional_manifest_path) => optional_manifest_path,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let manifest_path = clap_variant.manifest_path.clone();
        if clap_variant.color.is_none() {
            clap_variant.color = match Self::input_color(&context) {
                Ok(optional_color) => optional_color,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let color = clap_variant.color.clone();

        let args = Self {
            release,
            embed_abi,
            doc,
            no_abi,
            out_dir,
            manifest_path,
            color,
        };
        if let Err(err) = build::run(args).map(|_| ()) {
            interactive_clap::ResultFromCli::Err(Some(clap_variant), color_eyre::Report::msg(err))
        } else {
            interactive_clap::ResultFromCli::Ok(clap_variant)
        }
    }
}

impl BuildCommand {
    fn input_color(_context: &()) -> color_eyre::eyre::Result<Option<ColorPreference>> {
        Ok(None)
    }

    fn input_out_dir(
        _context: &(),
    ) -> color_eyre::eyre::Result<Option<crate::types::utf8_path_buf::Utf8PathBuf>> {
        Ok(None)
    }

    fn input_manifest_path(
        _context: &(),
    ) -> color_eyre::eyre::Result<Option<crate::types::utf8_path_buf::Utf8PathBuf>> {
        Ok(None)
    }
}

#[derive(Debug, EnumDiscriminants, Clone, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ColorPreference {
    Auto,
    Always,
    Never,
}

impl interactive_clap::ToCli for ColorPreference {
    type CliVariant = ColorPreference;
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

impl FromStr for ColorPreference {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(default_mode()),
            "always" => Ok(ColorPreference::Always),
            "never" => Ok(ColorPreference::Never),
            _ => Err(format!("invalid color preference: {}", s)),
        }
    }
}

fn default_mode() -> ColorPreference {
    match env::var("NO_COLOR") {
        Ok(v) if v != "0" => ColorPreference::Never,
        _ => {
            if atty::is(atty::Stream::Stderr) {
                ColorPreference::Always
            } else {
                ColorPreference::Never
            }
        }
    }
}

impl ColorPreference {
    pub fn as_str(&self) -> &str {
        match self {
            ColorPreference::Auto => "auto",
            ColorPreference::Always => "always",
            ColorPreference::Never => "never",
        }
    }

    pub fn apply(&self) {
        match self {
            ColorPreference::Auto => {
                default_mode().apply();
            }
            ColorPreference::Always => colored::control::set_override(true),
            ColorPreference::Never => colored::control::set_override(false),
        }
    }
}
