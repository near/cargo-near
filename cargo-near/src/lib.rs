use std::str::FromStr;
use std::{convert::Infallible, env};

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use interactive_clap::{InteractiveClap, ResultFromCli, ToCli, ToCliArgs};
use near_cli_rs::types::path_buf::PathBuf;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use strum_macros;

pub mod abi;
pub mod build;
mod cargo;
pub mod util;

// #[derive(Debug, EnumDiscriminants, InteractiveClap)]
// #[interactive_clap(bin_name = "cargo", version, about)]
// #[strum_discriminants(derive(EnumMessage, EnumIter))]
// #[non_exhaustive]
// pub enum Opts {
//     #[interactive_clap(name = "near", version, about)]
//     Near(Cmd),
// }

use near_cli_rs::GlobalContext;
type ConfigContext = (near_cli_rs::config::Config,);

#[derive(Debug, Clone, InteractiveClap)]
// #[interactive_clap(input_context = ConfigContext)]
// #[interactive_clap(output_context = CmdContext)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]

struct Cmd {
    /// Offline mode
    #[interactive_clap(long)]
    offline: bool,
    #[interactive_clap(subcommand)]
    top_level: NearCommand,
}

#[derive(Debug, Clone)]
struct CmdContext(crate::GlobalContext);

impl CmdContext {
    fn from_previous_context(
        previous_context: ConfigContext,
        scope: &<Cmd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(crate::GlobalContext {
            config: previous_context.0,
            offline: scope.offline,
        }))
    }
}

impl From<CmdContext> for crate::GlobalContext {
    fn from(item: CmdContext) -> Self {
        item.0
    }
}
// impl From<CmdContext> for () {
//     fn from(item: CmdContext) -> Self {
//         ()
//     }
// }
#[derive(Debug, InteractiveClap, EnumDiscriminants, Clone)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum NearCommand {
    /// Build a NEAR contract and optionally embed ABI
    #[interactive_clap(name = "build")]
    #[interactive_clap(subcommand)]
    Build(BuildCommand),
    /// Generates ABI for the contract
    #[interactive_clap(name = "abi")]
    #[interactive_clap(subcommand)]
    Abi(AbiCommand),
}

#[derive(Debug, InteractiveClap, Clone)]
#[interactive_clap(skip_default_from_cli)]
pub struct AbiCommand {
    // Include rustdocs in the ABI file
    #[interactive_clap(long)]
    pub doc: bool,
    /// Generate compact (minified) JSON
    #[interactive_clap(long)]
    pub compact_abi: bool,
    // Copy final artifacts to this directory
    #[interactive_clap(long, value_name = "PATH", value_parser= PathBuf::from_str)]
    #[interactive_clap(skip_default_input_arg)]
    pub out_dir: Option<PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[interactive_clap(long, value_name = "PATH", value_parser= PathBuf::from_str)]
    #[interactive_clap(skip_default_input_arg)]
    pub manifest_path: Option<PathBuf>,
    /// Coloring: auto, always, never
    #[interactive_clap(long, value_name = "WHEN", default_value = "auto")]
    #[interactive_clap(hide_default_value = true, hide_possible_values = true)]
    // #[clap(parse(try_from_str = ColorPreference::from_str))]
    #[interactive_clap(value_enum)]
    pub color: ColorPreference,
}

#[derive(Debug, InteractiveClap, Clone)]
#[interactive_clap(skip_default_from_cli)]
pub struct BuildCommand {
    // Build contract in release mode, with optimizations
    #[interactive_clap(short, long)]
    pub release: bool,
    /// Embed the ABI in the contract binary
    #[interactive_clap(long)]
    // #[arg(long = "embed-abi")]
    pub embed_abi: bool,
    /// Include rustdocs in the embedded ABI
    #[interactive_clap(long)]
    pub doc: bool,
    // Do not generate ABI for the contract
    #[interactive_clap(long, conflicts_with_all = &["doc", "embed_abi"])]
    pub no_abi: bool,
    // Copy final artifacts to this directory
    #[interactive_clap(long, value_name = "PATH", value_parser= PathBuf::from_str)]
    #[interactive_clap(skip_default_input_arg)]
    pub out_dir: Option<PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[interactive_clap(long, value_name = "PATH", value_parser= PathBuf::from_str)]
    #[interactive_clap(skip_default_input_arg)]
    pub manifest_path: Option<PathBuf>,
    /// Coloring: auto, always, never
    #[interactive_clap(long, value_name = "WHEN")]
    #[interactive_clap(default_value = "auto")]
    #[interactive_clap(hide_default_value = true, hide_possible_values = true)]
    #[interactive_clap(value_enum)]
    pub color: ColorPreference,
}
#[derive(Copy, Clone, Debug, ValueEnum, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, EnumMessage, EnumDiscriminants))]
pub enum ColorPreference {
    Auto,
    Always,
    Never,
}

impl ToCli for ColorPreference {
    type CliVariant = ColorPreference;
}

impl std::fmt::Display for ColorPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorPreference::Auto => write!(f, "auto"),
            ColorPreference::Always => write!(f, "always"),
            ColorPreference::Never => write!(f, "never"),
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

pub fn exec(cmd: NearCommand) -> anyhow::Result<()> {
    match cmd {
        NearCommand::Abi(args) => abi::run(args),
        NearCommand::Build(args) => build::run(args).map(|_| ()),
    }
}
