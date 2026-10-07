use crate::util;
use cargo_near_integration_tests::{
    MAX_RUST_VERSION, build_fn_with, common_root_for_test_projects_build, setup_tracing,
};
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

#[tokio::test]
#[named]
async fn test_build_wasmopt_oz() -> testresult::TestResult {
    setup_tracing();
    let oz_build = build_fn_with! {
        Opts: "--wasmopt-oz";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    // Leave the sources untouched so Cargo cannot hide a broken optimizer toggle
    // behind a rebuild. Both modes must always start from the same original Wasm.
    let project_dir = common_root_for_test_projects_build().join(function_name!());
    let manifest_path = project_dir.join("Cargo.toml");
    let cargo_wasm = project_dir
        .join("target/wasm32-unknown-unknown/release")
        .join(format!("{}.wasm", function_name!()));
    let cargo_modified = fs::metadata(&cargo_wasm)?.modified()?;
    let unoptimized_wasm = fs::read(&cargo_wasm)?;
    let rebuild = |no_wasmopt, wasmopt_oz, out_dir| -> testresult::TestResult<_> {
        let opts = cargo_near_build::BuildOpts::builder()
            .manifest_path(manifest_path.clone())
            .override_toolchain(MAX_RUST_VERSION)
            .no_wasmopt(no_wasmopt)
            .wasmopt_oz(wasmopt_oz)
            .maybe_out_dir(out_dir)
            .build();
        let artifact = cargo_near_build::build(opts)?;
        Ok((
            fs::read(&artifact.path)?,
            fs::metadata(artifact.path)?.modified()?,
        ))
    };
    let (default_wasm, _) = rebuild(false, false, None)?;
    assert_ne!(oz_build.wasm, default_wasm);
    assert_ne!(unoptimized_wasm, default_wasm);
    assert_eq!(rebuild(false, true, None)?.0, oz_build.wasm);
    assert_eq!(rebuild(false, false, None)?.0, default_wasm);
    assert_eq!(rebuild(true, false, None)?.0, unoptimized_wasm);
    assert_eq!(rebuild(true, true, None)?.0, unoptimized_wasm);
    assert_eq!(rebuild(false, false, None)?.0, default_wasm);

    // Repeated post-processing must leave identical output untouched.
    let default_modified = rebuild(false, false, None)?.1;
    assert_eq!(rebuild(false, false, None)?.1, default_modified);
    let oz_modified = rebuild(false, true, None)?.1;
    assert_eq!(rebuild(false, true, None)?.1, oz_modified);

    assert_eq!(fs::metadata(&cargo_wasm)?.modified()?, cargo_modified);
    assert_eq!(fs::read(&cargo_wasm)?, unoptimized_wasm);

    // Each destination must reflect the current options, regardless of the last
    // mode used for another --out-dir.
    let other_dir = project_dir.join("other-output");
    assert_eq!(
        rebuild(false, false, Some(other_dir.clone()))?.0,
        default_wasm
    );
    assert_eq!(rebuild(false, true, None)?.0, oz_build.wasm);
    assert_eq!(
        rebuild(true, false, Some(other_dir.clone()))?.0,
        unoptimized_wasm
    );
    assert_eq!(rebuild(false, true, Some(other_dir))?.0, oz_build.wasm);
    assert_eq!(rebuild(false, false, None)?.0, default_wasm);
    util::test_add(&default_wasm).await?;
    util::test_add(&oz_build.wasm).await?;

    Ok(())
}
