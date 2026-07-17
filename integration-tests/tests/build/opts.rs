use crate::util;
use cargo_near_integration_tests::{build_fn_with, setup_tracing};
use function_name::named;
use std::fs;

#[test]
#[named]
fn test_build_no_abi() -> testresult::TestResult {
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
async fn test_build_no_embed_abi() -> testresult::TestResult {
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
fn test_build_no_doc() -> testresult::TestResult {
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
fn test_build_opt_out_dir() -> testresult::TestResult {
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
async fn test_build_no_release() -> testresult::TestResult {
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

#[tokio::test]
#[named]
async fn test_build_custom_profile() -> testresult::TestResult {
    setup_tracing();
    let build_result = build_fn_with! {
        Opts: "--profile=release";
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

#[tokio::test]
#[named]
async fn test_build_abi_features_separate_from_wasm_features() -> testresult::TestResult {
    setup_tracing();
    let build_result = build_fn_with! {
        Cargo: "/templates/abi/_Cargo_features.toml";
        Opts: "--abi-features gated";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }

        #[cfg(feature = "gated")]
        pub fn gated_only(&self) -> bool {
            true
        }
    };

    // ABI should contain the gated_only function (abi_features has "gated")
    let abi_root = build_result.abi_root.unwrap();
    let function_names: Vec<&str> = abi_root
        .body
        .functions
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert!(
        function_names.contains(&"gated_only"),
        "ABI should include gated_only when --abi-features gated is used"
    );

    // WASM should NOT have gated_only (features does not have "gated")
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&build_result.wasm).await?;
    let outcome = contract.call("gated_only").view().await;
    assert!(
        outcome.is_err(),
        "WASM should NOT have gated_only compiled in when --features does not include gated"
    );

    // But add should still work
    util::test_add(&build_result.wasm).await?;

    Ok(())
}

#[tokio::test]
#[named]
async fn test_build_both_features_and_abi_features_for_different_targets() -> testresult::TestResult
{
    setup_tracing();
    let build_result = build_fn_with! {
        Cargo: "/templates/abi/_Cargo_features.toml";
        Opts: "--features gated";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }

        #[cfg(feature = "gated")]
        pub fn gated_only(&self) -> bool {
            true
        }
    };

    // ABI should contain gated_only (falls back to --features)
    let abi_root = build_result.abi_root.unwrap();
    let function_names: Vec<&str> = abi_root
        .body
        .functions
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert!(
        function_names.contains(&"gated_only"),
        "ABI should include gated_only"
    );

    // WASM should also have gated_only (--features gated)
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&build_result.wasm).await?;
    let outcome = contract.call("gated_only").view().await?;
    assert!(outcome.json::<bool>()?);

    Ok(())
}

/// Two builds of the same unchanged contract with different `--out-dir` values must not
/// recompile anything: nothing about the out-dir may leak into compile-time fingerprints.
/// Guards against the `CARGO_NEAR_ABI_PATH`-pointing-into-`--out-dir` regression, where
/// callers passing a fresh temp out-dir per invocation (e.g. near/intents' Makefile) got
/// the whole dependency graph rebuilt on every run.
#[test]
#[named]
fn test_build_twice_different_out_dirs_stays_fresh() -> testresult::TestResult {
    let out_dir1 = tempfile::tempdir()?;
    let out_dir2 = tempfile::tempdir()?;

    let _ = build_fn_with! {
        Opts: format!("--out-dir {}", out_dir1.path().display());
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    // the wasm produced by cargo (before wasm-opt / out-dir copies); any dirtied unit in
    // the dependency graph re-links this cdylib, so a stable mtime means full freshness
    let cargo_produced_wasm = cargo_near_integration_tests::common_root_for_test_projects_build()
        .join(function_name!())
        .join("target/wasm32-unknown-unknown/release")
        .join(format!("{}.wasm", function_name!()));
    assert!(
        cargo_produced_wasm.exists(),
        "cargo-produced wasm expected at {cargo_produced_wasm}"
    );
    let mtime_after_first = fs::metadata(&cargo_produced_wasm)?.modified()?;

    let _ = build_fn_with! {
        Opts: format!("--out-dir {}", out_dir2.path().display());
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };
    let mtime_after_second = fs::metadata(&cargo_produced_wasm)?.modified()?;

    assert_eq!(
        mtime_after_first, mtime_after_second,
        "changing --out-dir between otherwise identical builds must not recompile or re-link \
         anything, but the cargo-produced wasm was rewritten"
    );

    Ok(())
}
