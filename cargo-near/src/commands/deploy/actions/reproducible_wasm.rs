use near_cli_rs::commands::contract::deploy::initialize_mode::InitializeMode;

use crate::commands::build as build_command;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = context::Context)]
#[interactive_clap(skip_default_from_cli)]
pub struct DeployOpts {
    #[interactive_clap(flatten)]
    /// Build reproducible wasm step options
    build_command_opts: build_command::actions::reproducible_wasm::BuildOpts,
    /// Whether to check that code has been pushed to repository during docker build
    ///
    /// When flag is not specified, a sanity check that code being built has been pushed to remote repo
    /// is done before building+deploying a reproducible build.   
    /// This flag is used to skip this sanity check,
    ///
    /// Need for this was flag was discovered when deploying on Github into temporary accounts,
    /// created in CI for each pull request to test the PR's code.
    /// Merge commits were created for each CI run, which resided neither in base, nor target branches.
    /// These temporary per-PR contracts didn't need to be linked back to their source code,
    /// i.e. did not need reproducibility verification.
    #[interactive_clap(long)]
    #[interactive_clap(verbatim_doc_comment)]
    skip_git_remote_check: bool,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the target contract account ID to deploy to?
    contract_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    initialize: InitializeMode,
}

mod context {
    use crate::commands::build as build_command;

    #[derive(Debug, Clone)]
    pub struct Context(near_cli_rs::commands::contract::deploy::ContractFileContext);

    impl From<Context> for near_cli_rs::commands::contract::deploy::ContractFileContext {
        fn from(item: Context) -> Self {
            item.0
        }
    }

    impl Context {
        pub fn from_previous_context(
            previous_context: near_cli_rs::GlobalContext,
            scope: &<super::DeployOpts as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
        ) -> color_eyre::eyre::Result<Self> {
            let artifact = build_command::actions::reproducible_wasm::run(
                scope.build_command_opts.clone(),
                cargo_near_build::BuildContext::Deploy {
                    skip_git_remote_check: scope.skip_git_remote_check,
                },
            )?;
            let code = std::fs::read(&artifact.path).map_err(|err| {
                color_eyre::eyre::eyre!(
                    "accessing {} to read wasm contents: {}",
                    artifact.path,
                    err
                )
            })?;

            Ok(Self(
                near_cli_rs::commands::contract::deploy::ContractFileContext {
                    global_context: previous_context,
                    receiver_account_id: scope.contract_account_id.clone().into(),
                    signer_account_id: scope.contract_account_id.clone().into(),
                    code,
                },
            ))
        }
    }
}

/// this module is needed because of `#[interactive_clap(skip_default_input_arg)]`  
/// on `contract_account_id`
mod manual_input {
    impl super::DeployOpts {
        pub fn input_contract_account_id(
            context: &near_cli_rs::GlobalContext,
        ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
            near_cli_rs::common::input_signer_account_id_from_used_account_list(
                &context.config.credentials_home_dir,
                "What is the contract account ID?",
            )
        }
    }
}

/// this module is needed because of #[interactive_clap(skip_default_from_cli)]
/// on `DeployOpts`
mod manual_from_cli {
    use crate::commands::build as build_command;
    use near_cli_rs::commands::contract::deploy::initialize_mode::InitializeMode;

    impl interactive_clap::FromCli for super::DeployOpts {
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

            let build_command_opts =
                if let Some(cli_build_command_opts) = &clap_variant.build_command_opts {
                    build_command::actions::reproducible_wasm::BuildOpts::from(
                        cli_build_command_opts.clone(),
                    )
                } else {
                    build_command::actions::reproducible_wasm::BuildOpts::default()
                };

            if clap_variant.contract_account_id.is_none() {
                clap_variant.contract_account_id = match Self::input_contract_account_id(&context) {
                    Ok(Some(contract_account_id)) => Some(contract_account_id),
                    Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                    Err(err) => {
                        return interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                    }
                };
            }
            let contract_account_id = clap_variant
                .contract_account_id
                .clone()
                .expect("Unexpected error");

            let skip_git_remote_check = clap_variant.skip_git_remote_check;

            let new_context_scope = super::InteractiveClapContextScopeForDeployOpts {
                build_command_opts,
                contract_account_id,
                skip_git_remote_check,
            };

            let output_context =
                match super::context::Context::from_previous_context(context, &new_context_scope) {
                    Ok(new_context) => new_context,
                    Err(err) => {
                        return interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                    }
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
}
