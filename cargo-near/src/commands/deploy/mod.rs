use near_cli_rs::commands::contract::deploy::initialize_mode::InitializeMode;

use crate::commands::build_command;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Contract {
    #[interactive_clap(flatten)]
    /// Specify a build command args:
    build_command_args: build_command::BuildCommand,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    contract_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    initialize: InitializeMode,
}

#[derive(Debug, Clone)]
pub struct ContractContext(near_cli_rs::commands::contract::deploy::ContractFileContext);

impl ContractContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path = build_command::build::run(scope.build_command_args.clone())?.path;
        Ok(Self(
            near_cli_rs::commands::contract::deploy::ContractFileContext {
                global_context: previous_context,
                receiver_account_id: scope.contract_account_id.clone().into(),
                signer_account_id: scope.contract_account_id.clone().into(),
                code: std::fs::read(file_path)?,
            },
        ))
    }
}

impl From<ContractContext> for near_cli_rs::commands::contract::deploy::ContractFileContext {
    fn from(item: ContractContext) -> Self {
        item.0
    }
}

impl interactive_clap::FromCli for Contract {
    type FromCliContext = near_cli_rs::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        let build_command_args =
            if let Some(cli_build_command_args) = &clap_variant.build_command_args {
                build_command::BuildCommand {
                    no_release: cli_build_command_args.no_release,
                    no_abi: cli_build_command_args.no_abi,
                    no_embed_abi: cli_build_command_args.no_embed_abi,
                    no_doc: cli_build_command_args.no_doc,
                    out_dir: cli_build_command_args.out_dir.clone(),
                    manifest_path: cli_build_command_args.manifest_path.clone(),
                    color: cli_build_command_args.color.clone(),
                }
            } else {
                build_command::BuildCommand::default()
            };

        if clap_variant.contract_account_id.is_none() {
            clap_variant.contract_account_id = match Self::input_contract_account_id(&context) {
                Ok(Some(contract_account_id)) => Some(contract_account_id),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let contract_account_id = clap_variant
            .contract_account_id
            .clone()
            .expect("Unexpected error");

        let new_context_scope = InteractiveClapContextScopeForContract {
            build_command_args,
            contract_account_id,
        };

        let output_context =
            match ContractContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };

        match InitializeMode::from_cli(clap_variant.initialize.take(), output_context.into()) {
            interactive_clap::ResultFromCli::Ok(initialize) => {
                clap_variant.initialize = Some(initialize);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_initialize) => {
                clap_variant.initialize = optional_initialize;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_initialize, err) => {
                clap_variant.initialize = optional_initialize;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

impl Contract {
    pub fn input_contract_account_id(
        context: &near_cli_rs::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        near_cli_rs::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}
