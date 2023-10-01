use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod abi_command;
pub mod build_command;

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
