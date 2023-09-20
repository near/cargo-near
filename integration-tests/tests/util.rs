use near_abi::AbiRoot;
use near_abi::{AbiBorshParameter, AbiJsonParameter, AbiParameters};
use serde_json::json;
use workspaces::prelude::DevAccountDeployer;

/// Utility method to test that the `add` function is available and works as intended
pub async fn test_add(wasm: &[u8]) -> cargo_near::CliResult {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(wasm).await?;
    let outcome = contract
        .call(&worker, "add")
        .args_json(json!({
            "a": 2u32,
            "b": 3u32,
        }))?
        .view()
        .await?;
    assert_eq!(outcome.json::<u32>()?, 5);
    Ok(())
}

pub async fn fetch_contract_abi(wasm: &[u8]) -> color_eyre::eyre::Result<AbiRoot> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(wasm).await?;
    let outcome = contract.call(&worker, "__contract_abi").view().await?;
    let outcome_json = zstd::decode_all(outcome.result.as_slice())?;
    Ok(serde_json::from_slice::<AbiRoot>(&outcome_json)?)
}

pub trait AsBorshSchema {
    fn borsh_schemas(&self) -> color_eyre::eyre::Result<&Vec<AbiBorshParameter>>;
}

impl AsBorshSchema for AbiParameters {
    fn borsh_schemas(&self) -> color_eyre::eyre::Result<&Vec<AbiBorshParameter>> {
        if let AbiParameters::Borsh { args } = &self {
            Ok(args)
        } else {
            color_eyre::eyre::bail!("Expected Borsh serialization type, but got {:?}", self)
        }
    }
}

pub trait AsJsonSchema {
    fn json_schemas(&self) -> color_eyre::eyre::Result<&Vec<AbiJsonParameter>>;
}

impl AsJsonSchema for AbiParameters {
    fn json_schemas(&self) -> color_eyre::eyre::Result<&Vec<AbiJsonParameter>> {
        if let AbiParameters::Json { args } = &self {
            Ok(args)
        } else {
            color_eyre::eyre::bail!("Expected JSON serialization type, but got {:?}", self)
        }
    }
}
