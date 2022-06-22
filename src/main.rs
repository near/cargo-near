use cargo::manifest::CargoManifestPath;
use clap::{AppSettings, Args, Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod abi;
mod cargo;
mod util;

#[derive(Debug, Parser)]
#[clap(bin_name = "cargo")]
pub(crate) enum Opts {
    #[clap(name = "near")]
    #[clap(setting = AppSettings::DeriveDisplayOrder)]
    Near(NearArgs),
}

#[derive(Debug, Args)]
pub(crate) struct NearArgs {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generates ABI for the contract
    #[clap(name = "abi")]
    Abi(AbiCommand),
}

#[derive(Debug, clap::Args)]
#[clap(name = "abi")]
pub struct AbiCommand {
    /// Path to the `Cargo.toml` of the contract to build
    #[clap(long, parse(from_os_str))]
    manifest_path: Option<PathBuf>,
}

fn main() {
    env_logger::init();

    let Opts::Near(args) = Opts::parse();
    match exec(args.cmd) {
        Ok(()) => {}
        Err(err) => {
            eprintln!(
                "{} {}",
                "ERROR:".bright_red().bold(),
                format!("{:?}", err).bright_red()
            );
            std::process::exit(1);
        }
    }
}

fn exec(cmd: Command) -> anyhow::Result<()> {
    match &cmd {
        Command::Abi(abi) => {
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
