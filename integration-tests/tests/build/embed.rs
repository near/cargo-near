use crate::build::util;
use cargo_near_integration_tests::{build_fn, build_fn_with};
use function_name::named;
use workspaces::prelude::DevAccountDeployer;

#[tokio::test]
#[named]
async fn test_build_embed_abi() -> anyhow::Result<()> {
    let build_result = build_fn_with! {
        Opts: "--embed-abi";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let abi_root = build_result.abi_root.unwrap();
    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.name, "add");

    let (add, actual_abi) = tokio::join!(
        util::test_add(&build_result.wasm),
        util::fetch_contract_abi(&build_result.wasm)
    );
    add?;
    assert_eq!(abi_root, actual_abi?);

    Ok(())
}

#[tokio::test]
#[named]
async fn test_build_no_embed_abi() -> anyhow::Result<()> {
    let build_result = build_fn! {
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&build_result.wasm).await?;
    let outcome = contract.call(&worker, "__contract_abi").view().await;
    outcome.unwrap_err();

    Ok(())
}
