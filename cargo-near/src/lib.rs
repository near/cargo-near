use std::{env, str::FromStr};

use camino::Utf8PathBuf;
use clap::{AppSettings, Args, Parser, Subcommand};

pub mod abi;
pub mod build;
mod cargo;
pub mod util;

#[derive(Debug, Parser)]
#[clap(bin_name = "cargo", version, about)]
pub enum Opts {
    #[clap(name = "near", version, about)]
    #[clap(setting = AppSettings::DeriveDisplayOrder)]
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
}

#[derive(Debug, clap::Args)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct AbiCommand {
    /// Include rustdocs in the ABI file
    #[clap(long)]
    pub doc: bool,
    /// Generate compact (minified) JSON
    #[clap(long)]
    pub compact_abi: bool,
    /// Copy final artifacts to this directory
    #[clap(long, parse(from_str), value_name = "PATH")]
    pub out_dir: Option<Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[clap(long, parse(from_str), value_name = "PATH")]
    pub manifest_path: Option<Utf8PathBuf>,
    /// Coloring: auto, always, never
    #[clap(long, value_name = "WHEN")]
    #[clap(default_value = "auto", possible_values = &["auto", "always", "never"])]
    #[clap(hide_default_value = true, hide_possible_values = true)]
    #[clap(parse(try_from_str = ColorPreference::from_str))]
    pub color: ColorPreference,
}

#[derive(Debug, clap::Args)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct BuildCommand {
    /// Build contract in release mode, with optimizations
    #[clap(short, long)]
    pub release: bool,
    /// Embed the ABI in the contract binary
    #[clap(long)]
    pub embed_abi: bool,
    /// Include rustdocs in the embedded ABI
    #[clap(long)]
    pub doc: bool,
    /// Do not generate ABI for the contract
    #[clap(long, conflicts_with_all = &["doc", "embed-abi"])]
    pub no_abi: bool,
    /// Copy final artifacts to this directory
    #[clap(long, parse(from_str), value_name = "PATH")]
    pub out_dir: Option<Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[clap(long, parse(from_str), value_name = "PATH")]
    pub manifest_path: Option<Utf8PathBuf>,
    /// Coloring: auto, always, never
    #[clap(long, value_name = "WHEN")]
    #[clap(default_value = "auto", possible_values = &["auto", "always", "never"])]
    #[clap(hide_default_value = true, hide_possible_values = true)]
    #[clap(parse(try_from_str = ColorPreference::from_str))]
    pub color: ColorPreference,
}

#[derive(Copy, Clone, Debug)]
pub enum ColorPreference {
    Always,
    Never,
}

impl FromStr for ColorPreference {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => match env::var("NO_COLOR") {
                Ok(v) if v != "0" => Ok(ColorPreference::Never),
                _ => {
                    if atty::is(atty::Stream::Stderr) {
                        Ok(ColorPreference::Always)
                    } else {
                        Ok(ColorPreference::Never)
                    }
                }
            },
            "always" => Ok(ColorPreference::Always),
            "never" => Ok(ColorPreference::Never),
            _ => Err(format!("invalid color preference: {}", s)),
        }
    }
}

impl ColorPreference {
    pub fn as_str(&self) -> &str {
        match self {
            ColorPreference::Always => "always",
            ColorPreference::Never => "never",
        }
    }

    pub fn apply(&self) {
        match self {
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
