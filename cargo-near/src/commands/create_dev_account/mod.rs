use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct CreateAccount {
    #[interactive_clap(subcommand)]
    account_actions: CreateAccountMethod,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you cover the costs of account creation?
pub enum CreateAccountMethod {
    #[strum_discriminants(strum(
        message = "use-random-account-id     - I would like to create a random account"
    ))]
    /// I would like to create a random account
    UseRandomAccountId,
    #[strum_discriminants(strum(
        message = "use-specific-account-id   - I would like to create a specific account"
    ))]
    /// I would like to create a specific account
    UseSpecificAccountId(
        near_cli_rs::commands::account::create_account::sponsor_by_faucet_service::NewAccount,
    ),
}
