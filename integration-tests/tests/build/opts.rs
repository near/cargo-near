use crate::build::util;
use cargo_near_integration_tests::{build_fn_with, setup_tracing};
use function_name::named;
use std::fs;

#[test]
#[named]
fn test_build_no_abi() -> cargo_near::CliResult {
    let build_result = build_fn_with! {
        Opts: "--no-abi";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };
    assert!(build_result.abi_root.is_none());
    assert!(build_result.abi_compressed.is_none());

    Ok(())
}

#[tokio::test]
#[named]
async fn test_build_no_embed_abi() -> cargo_near::CliResult {
    let build_result = build_fn_with! {
        Opts: "--no-embed-abi";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&build_result.wasm).await?;
    let outcome = contract.call("__contract_abi").view().await;
    outcome.unwrap_err();

    Ok(())
}

#[test]
#[named]
fn test_build_no_doc() -> cargo_near::CliResult {
    let build_result = build_fn_with! {
        Opts: "--no-doc";
        Code:
        /// Adds `a` and `b`.
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let abi_root = build_result.abi_root.unwrap();
    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[0];
    assert!(function.doc.is_none());

    Ok(())
}

#[test]
#[named]
fn test_build_opt_out_dir() -> cargo_near::CliResult {
    let out_dir = tempfile::tempdir()?;
    let build_result = build_fn_with! {
        Opts: format!("--out-dir {}", out_dir.path().display());
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let abi_json = fs::read(
        out_dir
            .path()
            .join(format!("{}_abi.json", function_name!())),
    )?;
    assert_eq!(
        build_result.abi_root.unwrap(),
        serde_json::from_slice(&abi_json)?
    );

    Ok(())
}

#[tokio::test]
#[named]
async fn test_build_no_release() -> cargo_near::CliResult {
    setup_tracing();
    let build_result = build_fn_with! {
        Opts: "--no-release";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let abi_root = build_result.abi_root.unwrap();
    assert_eq!(abi_root.body.functions.len(), 2);
    assert!(build_result.abi_compressed.is_some());
    util::test_add(&build_result.wasm).await?;

    Ok(())
}
