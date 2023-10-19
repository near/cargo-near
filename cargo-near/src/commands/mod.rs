use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod abi_command;
pub mod build_command;
pub mod create_dev_account;
pub mod deploy;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum NearCommand {
    #[strum_discriminants(strum(
        message = "build               -  Build a NEAR contract and optionally embed ABI"
    ))]
    /// Build a NEAR contract and optionally embed ABI
    Build(self::build_command::BuildCommand),
    #[strum_discriminants(strum(
        message = "abi                 -  Generates ABI for the contract"
    ))]
    /// Generates ABI for the contract
    Abi(self::abi_command::AbiCommand),
    #[strum_discriminants(strum(
        message = "create-dev-account  -  Create a development account using the faucet service sponsor to cover
                         the cost of creating an account (testnet only for now).
                         To create an account on another network, you need to use \"near-cli-rs\":
                         https://github.com/near/near-cli-rs"
    ))]
    /// Create a development account using the faucet service sponsor to cover the cost of creating an account (testnet only for now)
    CreateDevAccount(self::create_dev_account::CreateAccount),
    #[strum_discriminants(strum(message = "deploy              -  Add a new contract code"))]
    /// Add a new contract code
    Deploy(self::deploy::Contract),
}
