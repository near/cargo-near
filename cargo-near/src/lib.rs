use clap::{AppSettings, Args, Parser, Subcommand};
use std::path::PathBuf;

mod abi;
mod build;
mod cargo;
mod util;

#[derive(Debug, Parser)]
#[clap(bin_name = "cargo", version, author, about)]
pub enum Opts {
    #[clap(
        name = "near",
        version,
        author,
        about,
        setting = AppSettings::DeriveDisplayOrder,
    )]
    Near(NearArgs),
}

#[derive(Debug, Args)]
pub struct NearArgs {
    #[clap(subcommand)]
    pub cmd: NearCommand,
}

#[derive(Debug, Subcommand)]
pub enum NearCommand {
    /// Generates ABI for the contract
    #[clap(name = "abi")]
    Abi(AbiCommand),
    /// Build a NEAR contract and optionally embed ABI
    #[clap(name = "build")]
    Build(BuildCommand),
}

#[derive(Debug, clap::Args)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub struct AbiCommand {
    /// Include rustdocs in the ABI file
    #[clap(long)]
    pub doc: bool,
    /// Copy final artifacts to the this directory
    #[clap(long, parse(from_os_str), value_name = "PATH")]
    pub out_dir: Option<PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[clap(long, parse(from_os_str), value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
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
    #[clap(long, requires = "embed-abi")]
    pub doc: bool,
    /// Do not generate ABI for the contract
    #[clap(long, conflicts_with_all = &["doc", "embed-abi"])]
    pub no_abi: bool,
    /// Copy final artifacts to the this directory
    #[clap(long, parse(from_os_str), value_name = "PATH")]
    pub out_dir: Option<PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    #[clap(long, parse(from_os_str), value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
}

pub fn exec(cmd: NearCommand) -> anyhow::Result<()> {
    match cmd {
        NearCommand::Abi(args) => abi::run(args),
        NearCommand::Build(args) => build::run(args),
    }
}
