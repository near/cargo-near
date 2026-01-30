use std::str::FromStr;

use color_eyre::eyre::ContextCompat;
use names::Generator;

use near_cli_rs::commands::account::create_account::sponsor_by_faucet_service::{
    NewAccountContext, add_key, before_creating_account, network,
};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = RandomAccountContext)]
pub struct RandomAccount {
    #[interactive_clap(subcommand)]
    access_key_mode: add_key::AccessKeyMode,
}

#[derive(Clone)]
pub struct RandomAccountContext(NewAccountContext);

impl RandomAccountContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        _scope: &<RandomAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let credentials_home_dir = previous_context.config.credentials_home_dir.clone();
        let random_account_id = random_account_id(&previous_context.config.network_connection)?;

        let on_before_creating_account_callback: network::OnBeforeCreatingAccountCallback =
            std::sync::Arc::new({
                move |network_config, new_account_id, public_key, storage_message| {
                    before_creating_account(
                        network_config,
                        new_account_id,
                        public_key,
                        &credentials_home_dir,
                        storage_message,
                        previous_context.verbosity,
                    )
                }
            });

        Ok(Self(NewAccountContext {
            config: previous_context.config,
            new_account_id: random_account_id,
            on_before_creating_account_callback,
        }))
    }
}

impl From<RandomAccountContext> for NewAccountContext {
    fn from(item: RandomAccountContext) -> Self {
        item.0
    }
}

pub fn random_account_id(
    networks: &linked_hash_map::LinkedHashMap<String, near_cli_rs::config::NetworkConfig>,
) -> color_eyre::eyre::Result<near_cli_rs::types::account_id::AccountId> {
    loop {
        let mut generator = Generator::default();
        let random_name = generator.next().wrap_err("Random name generator error")?;
        let account_id =
            near_cli_rs::types::account_id::AccountId::from_str(&format!("{random_name}.testnet"))?;
        if !near_cli_rs::common::is_account_exist(networks, account_id.clone().into())? {
            return Ok(account_id);
        }
    }
}
