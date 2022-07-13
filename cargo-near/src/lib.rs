use cargo::manifest::CargoManifestPath;
use clap::{AppSettings, Args, Parser, Subcommand};
use std::path::PathBuf;

mod abi;
mod cargo;
mod util;

#[derive(Debug, Parser)]
#[clap(bin_name = "cargo")]
pub enum Opts {
    #[clap(name = "near")]
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
    /// Generates ABI for the contract
    #[clap(name = "abi")]
    Abi(AbiCommand),
}

#[derive(Debug, clap::Args)]
#[clap(name = "abi")]
pub struct AbiCommand {
    /// Path to the `Cargo.toml` of the contract to build
    #[clap(long, parse(from_os_str))]
    pub manifest_path: Option<PathBuf>,
}

pub fn exec(cmd: NearCommand) -> anyhow::Result<()> {
    match &cmd {
        NearCommand::Abi(abi) => {
            let manifest_path = abi
                .manifest_path
                .clone()
                .unwrap_or_else(|| "Cargo.toml".into());
            let manifest_path = CargoManifestPath::try_from(manifest_path)?;

            let result = abi::execute(&manifest_path)?;
            println!("ABI successfully generated at {}", result.path.display());
            Ok(())
        }
    }
}
