use std::env;
use std::str::FromStr;

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};

pub mod abi;
pub mod build;
mod cargo;
pub mod deploy;
pub mod new;
pub mod util;

#[derive(Debug, Parser)]
#[clap(bin_name = "cargo", version, about)]
pub enum Opts {
    #[clap(name = "near", version, about)]
    Near(NearArgs),
}

#[derive(Debug, Args)]
pub struct NearArgs {
    #[clap(subcommand)]
    pub cmd: NearCommand,
}

#[derive(Debug, Subcommand)]
pub enum NearCommand {
    /// Build a NEAR contract and optionally embed ABI
    #[clap(name = "build")]
    Build(BuildCommand),
    /// Generates ABI for the contract
    #[clap(name = "abi")]
    Abi(AbiCommand),
    // Deploy a NEAR contract
    #[clap(name = "deploy")]
    Deploy(DeployCommand),
    // Create a new NEAR contract
    #[clap(name = "new")]
    New(NewCommand),
}
#[derive(Debug, clap::Args)]
pub struct DeployCommand {
    /// Include rustdocs in the ABI file
    // #[clap(long)]
    // pub account_id: String,
    /// Generate compact (minified) JSON
    // #[clap(long)]
    // pub compact_abi: bool,
    ///  Path of the contract to deploy
    #[clap(
        long,
        value_name = "PATH",
        default_value = "contract/target/near/hello_near.wasm"
    )]
    pub path: Utf8PathBuf,
    // /// Path to the `Cargo.toml` of the contract to build
    // #[clap(long, value_name = "PATH")]
    // pub manifest_path: Option<Utf8PathBuf>,
    #[clap(default_value = "testnet")]
    #[arg(value_enum)]
    pub network: Network,
    #[clap(long, value_name = "WHEN")]
    #[clap(default_value = "auto")]
    #[clap(hide_default_value = true, hide_possible_values = true)]
    #[arg(value_enum)]
    pub color: ColorPreference,
}
#[derive(Debug, clap::Args)]
pub struct NewCommand {
    #[clap(long, value_name = "PROJECT_NAME")]
    #[clap(default_value = "example_contract")]
    project_dir: Utf8PathBuf,
    #[clap(long, value_name = "WHEN")]
    #[clap(default_value = "auto")]
    #[clap(hide_default_value = true, hide_possible_values = true)]
    #[arg(value_enum)]
    pub color: ColorPreference,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Network {
    Testnet,
    // Betanet,
    Mainnet,
    // Localnet,
}

#[derive(Debug, clap::Args)]
pub struct AbiCommand {
    /// Include rustdocs in the ABI file
    #[clap(long)]
    pub doc: bool,
    /// Generate compact (minified) JSON
    #[clap(long)]
    pub compact_abi: bool,
    /// Copy final artifacts to this directory
    #[clap(long, value_name = "PATH")]
    pub out_dir: Option<Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[clap(long, value_name = "PATH")]
    pub manifest_path: Option<Utf8PathBuf>,
    /// Coloring: auto, always, never
    #[clap(long, value_name = "WHEN")]
    #[clap(default_value = "auto")]
    #[clap(hide_default_value = true, hide_possible_values = true)]
    #[arg(value_enum)]
    pub color: ColorPreference,
}

#[derive(Debug, clap::Args)]
pub struct BuildCommand {
    /// Build contract in release mode, with optimizations
    #[clap(short, long)]
    pub release: bool,
    /// Embed the ABI in the contract binary
    #[clap(long)]
    // #[arg(long = "embed-abi")]
    pub embed_abi: bool,
    /// Include rustdocs in the embedded ABI
    #[clap(long)]
    pub doc: bool,
    /// Do not generate ABI for the contract
    #[clap(long, conflicts_with_all = &["doc", "embed_abi"])]
    pub no_abi: bool,
    /// Copy final artifacts to this directory
    #[clap(long, value_name = "PATH")]
    pub out_dir: Option<Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[clap(long, value_name = "PATH")]
    pub manifest_path: Option<Utf8PathBuf>,
    /// Coloring: auto, always, never
    #[clap(long, value_name = "WHEN")]
    #[clap(default_value = "auto")]
    #[clap(hide_default_value = true, hide_possible_values = true)]
    #[arg(value_enum)]
    pub color: ColorPreference,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ColorPreference {
    Auto,
    Always,
    Never,
}

impl FromStr for ColorPreference {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(get_default_color_preference()),
            "always" => Ok(ColorPreference::Always),
            "never" => Ok(ColorPreference::Never),
            _ => Err(format!("invalid color preference: {}", s)),
        }
    }
}

fn get_default_color_preference() -> ColorPreference {
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
                get_default_color_preference().apply();
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
        NearCommand::Deploy(args) => deploy::run(args),
        NearCommand::New(args) => new::run(args),
    }
}
