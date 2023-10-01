use crate::build::util;
use cargo_near_integration_tests::build_fn_with;
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

#[test]
#[named]
fn test_build_opt_doc() -> cargo_near::CliResult {
    let build_result = build_fn_with! {
        Opts: "--doc";
        Code:
        /// Adds `a` and `b`.
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let abi_root = build_result.abi_root.unwrap();
    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.doc.as_ref().unwrap(), " Adds `a` and `b`.");

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
// TODO: remove ignore after near-workspaces-rs supports Rust 1.70+
// https://github.com/near/cargo-near/issues/104
#[ignore]
async fn test_build_opt_release() -> cargo_near::CliResult {
    let build_result = build_fn_with! {
        Opts: "--release";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let abi_root = build_result.abi_root.unwrap();
    assert_eq!(abi_root.body.functions.len(), 1);
    assert!(build_result.abi_compressed.is_none());
    util::test_add(&build_result.wasm).await?;

    Ok(())
}

#[tokio::test]
#[named]
// TODO: remove ignore after near-workspaces-rs supports Rust 1.70+
// https://github.com/near/cargo-near/issues/104
#[ignore]
async fn test_build_opt_doc_embed() -> cargo_near::CliResult {
    let build_result = build_fn_with! {
        Opts: "--doc --embed-abi";
        Code:
        /// Adds `a` and `b`.
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let mut abi_root = build_result.abi_root.unwrap();
    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.doc.as_ref().unwrap(), " Adds `a` and `b`.");

    // WASM hash is not included in the embedded ABI
    abi_root.metadata.wasm_hash = None;
    assert_eq!(
        util::fetch_contract_abi(&build_result.wasm).await?,
        abi_root
    );

    Ok(())
}

#[test]
#[named]
// TODO: remove ignore after near-workspaces-rs supports Rust 1.70+
// https://github.com/near/cargo-near/issues/104
#[ignore]
fn test_build_opt_no_abi_doc() -> cargo_near::CliResult {
    fn run_test() -> cargo_near::CliResult {
        build_fn_with! {
            Opts: "--no-abi --doc";
            Code:
            /// Adds `a` and `b`.
            pub fn add(&self, a: u32, b: u32) -> u32 {
                a + b
            }
        };
        Ok(())
    }
    assert!(run_test()
        .unwrap_err()
        .to_string()
        .contains("The argument '--no-abi' cannot be used with '--doc'"));

    Ok(())
}

#[test]
#[named]
// TODO: remove ignore after near-workspaces-rs supports Rust 1.70+
// https://github.com/near/cargo-near/issues/104
#[ignore]
fn test_build_opt_no_abi_embed() -> cargo_near::CliResult {
    fn run_test() -> cargo_near::CliResult {
        build_fn_with! {
            Opts: "--no-abi --embed-abi";
            Code:
            /// Adds `a` and `b`.
            pub fn add(&self, a: u32, b: u32) -> u32 {
                a + b
            }
        };
        Ok(())
    }
    assert!(run_test()
        .unwrap_err()
        .to_string()
        .contains("The argument '--no-abi' cannot be used with '--embed-abi'"));

    Ok(())
}
