use anyhow::Ok;
use color_eyre::eyre::{ContextCompat, WrapErr};
use near_cli_rs::common::JsonRpcClientExt;
use near_cli_rs::common::RpcQueryResponseExt;
use near_cli_rs::types::account_id;
use near_cli_rs::types::public_key;
use near_sdk::env;
use near_sdk::Promise;

use std::str::FromStr;

use crate::{abi, util, DeployCommand};
pub fn run(args: DeployCommand) -> anyhow::Result<()> {
    args.color.apply();

    let config = near_cli_rs::config::Config::default();

    let code = std::fs::read(&args.path).expect(&format!(
        "Failed to open or read the file: {:?}.",
        &args.path
    ));
    let action = near_primitives::transaction::Action::DeployContract(
        near_primitives::transaction::DeployContractAction { code },
    );
    // Promise::new("subaccount.example.near".parse().unwrap())
    //     .create_account()
    //     .add_full_access_key(env::signer_account_pk())
    //     .transfer(250_000_000_000_000_000_000_000); // 2.5e23yN, 0.25N

    // loop {
    //     if !near_cli_rs::common::is_account_exist(args.account_id.clone().into()) {
    //         println!(
    //             "\nThe account <{}> does not yet exist.",
    //             &deploy_to_account_id
    //         );
    //         #[derive(strum_macros::Display)]
    //         enum ConfirmOptions {
    //             #[strum(to_string = "Yes, I want to enter a new account name.")]
    //             Yes,
    //             #[strum(to_string = "No, I want to use this account name.")]
    //             No,
    //         }
    //         let select_choose_input = Select::new(
    //             "Do you want to enter a new component deployment account name?",
    //             vec![ConfirmOptions::Yes, ConfirmOptions::No],
    //         )
    //         .prompt()?;
    //         if let ConfirmOptions::No = select_choose_input {
    //             return Ok(Some(deploy_to_account_id));
    //         }
    //     } else {
    //         return Ok(Some(deploy_to_account_id));
    //     }
    // }

    let network = std::env::var_os("NEAR_NETWORK_CONNECTION");
    let signer_id = std::env::var_os("NEAR_SIGNER_ACCOUNT_ID");
    let public_key = std::env::var_os("NEAR_SIGNER_ACCOUNT_PUBLIC_KEY");
    let private_key = std::env::var_os("NEAR_SIGNER_ACCOUNT_PRIVATE_KEY");
    if let (Some(network), Some(signer_id), Some(public_key), Some(private_key)) =
        (network, signer_id, public_key, private_key)
    {
        util::print_step("Parsing env variables...");
        let public_key: near_crypto::PublicKey =
            near_crypto::PublicKey::from_str(public_key.to_str().unwrap())?;
        let private_key: near_crypto::SecretKey =
            near_crypto::SecretKey::from_str(private_key.to_str().unwrap())?;
        let signer_id: near_primitives::types::AccountId =
            near_primitives::types::AccountId::from_str(signer_id.to_str().unwrap())?;
        let network_config = config
            .network_connection
            .get(network.to_str().unwrap())
            .unwrap_or_else(|| panic!("Invalid NEAR_NETWORK_CONNECTION: {:?}", network));
        let rpc_query_response = network_config.json_rpc_client().blocking_call_view_access_key(
                    &signer_id,
                    &public_key,
                    near_primitives::types::BlockReference::latest()
                ).wrap_err(
                    "Cannot sign a transaction due to an error while fetching the most recent nonce value",
                ).unwrap();
        let (nonce, block_hash, _block_height) = (
            rpc_query_response
                .access_key_view()
                .wrap_err("Error current_nonce")
                .unwrap()
                .nonce
                + 1,
            rpc_query_response.block_hash,
            rpc_query_response.block_height,
        );

        let unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: public_key.clone(),
            block_hash,
            nonce,
            signer_id: signer_id.clone(),
            receiver_id: signer_id,
            actions: vec![action],
        };

        let signature = private_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());

        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        eprintln!("\nYour transaction was signed successfully.");
        eprintln!("Public key: {}", public_key);
        eprintln!("Signature: {}", signature);
        let transaction_info = network_config
            .json_rpc_client()
            .blocking_call(
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                    signed_transaction: signed_transaction.clone(),
                },
            )
            .wrap_err("Error broadcasting transaction")
            .unwrap();

        eprintln!("Transaction Id: {}", signed_transaction.get_hash());
        near_cli_rs::common::print_transaction_status(&transaction_info, network_config);
        return Ok(());
    } else {
        util::print_error("Missing environment variables");
    }

    util::print_step("Trying to find keychain in home dir...");
    println!("home dir: {:?}", config.credentials_home_dir);
    let network = {
        match args.network {
            crate::Network::Testnet => "testnet".to_string(),
            crate::Network::Mainnet => "mainnet".to_string(),
        }
    };

    let data_path: std::path::PathBuf = {
        let path = config.credentials_home_dir;
        let network_config = config
            .network_connection
            .get(&network)
            .unwrap_or_else(|| panic!("Invalid NEAR_NETWORK_CONNECTION: {:?}", network));
        let dir_name = network.clone();
        path.push(&dir_name);

        path.push(file_name);
        if path.exists() {
            path
        } else {
            let access_key_list = network_config
                .json_rpc_client()
                .blocking_call_view_access_key_list(
                    &signer_id,
                    near_primitives::types::Finality::Final.into(),
                )
                .wrap_err_with(|| {
                    format!(
                        "Failed to fetch access KeyList for {}",
                        previous_context.prepopulated_transaction.signer_id
                    )
                })?
                .access_key_list_view()?;
            let mut path = std::path::PathBuf::from(
                &previous_context.global_context.config.credentials_home_dir,
            );
            path.push(dir_name);
            path.push(
                &previous_context
                    .prepopulated_transaction
                    .signer_id
                    .to_string(),
            );
            let mut data_path = std::path::PathBuf::new();
            'outer: for access_key in access_key_list.keys {
                let account_public_key = access_key.public_key.to_string();
                let is_full_access_key: bool = match &access_key.access_key.permission {
                    near_primitives::views::AccessKeyPermissionView::FullAccess => true,
                    near_primitives::views::AccessKeyPermissionView::FunctionCall {
                        allowance: _,
                        receiver_id: _,
                        method_names: _,
                    } => false,
                };
                let dir = path
                        .read_dir()
                        .wrap_err("There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain.")?;
                for entry in dir {
                    if let Ok(entry) = entry {
                        if entry
                            .path()
                            .file_stem()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .contains(account_public_key.rsplit(':').next().unwrap())
                            && is_full_access_key
                        {
                            data_path.push(entry.path());
                            break 'outer;
                        }
                    } else {
                        return Err(color_eyre::Report::msg(
                                "There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain."
                            ));
                    };
                }
            }
            data_path
        }
    };
    let data = std::fs::read_to_string(&data_path).wrap_err("Access key file not found!")?;

    // util::print_success(&format!("Deployed to {}", args.account_id));
    Ok(())
}
