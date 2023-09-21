pub use near_cli_rs::CliResult;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod abi_command;
pub mod build_command;
mod common;
pub mod types;
mod util;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Cmd {
    #[interactive_clap(subcommand)]
    opts: Opts,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// Near
pub enum Opts {
    #[strum_discriminants(strum(message = "near"))]
    /// Which cargo extension do you want to use?
    Near(NearArgs),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct NearArgs {
    #[interactive_clap(subcommand)]
    pub cmd: NearCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum NearCommand {
    #[strum_discriminants(strum(
        message = "build   -   Build a NEAR contract and optionally embed ABI"
    ))]
    /// Build a NEAR contract and optionally embed ABI
    Build(self::build_command::BuildCommand),
    #[strum_discriminants(strum(message = "abi     -   Generates ABI for the contract"))]
    /// Generates ABI for the contract
    Abi(self::abi_command::AbiCommand),
}
