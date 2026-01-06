use cargo_near_integration_tests::{
    common_root_for_test_projects_build, generate_abi_fn_with, setup_tracing,
};
use function_name::named;
use std::fs;

#[test]
#[named]
fn test_abi_no_doc() -> testresult::TestResult {
    setup_tracing();
    let abi_root = generate_abi_fn_with! {
        Opts: "--no-doc";
        Code:
        /// Adds `a` and `b`.
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[0];
    assert!(function.doc.is_none());

    Ok(())
}

#[test]
#[named]
fn test_abi_opt_compact_abi() -> testresult::TestResult {
    setup_tracing();
    generate_abi_fn_with! {
        Opts: "--compact-abi";
        Code:
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    let workspace_dir = common_root_for_test_projects_build();
    let expected_target_dir = workspace_dir
        .join(function_name!())
        .join("target")
        .join("near");

    tracing::info!("expected target dir: {:?}", expected_target_dir);

    let abi_json =
        fs::read_to_string(expected_target_dir.join(format!("{}_abi.json", function_name!())))?;

    assert_eq!(minifier::json::minify(&abi_json).to_string(), abi_json);

    Ok(())
}

#[test]
#[named]
fn test_abi_opt_out_dir() -> testresult::TestResult {
    setup_tracing();
    let out_dir = tempfile::tempdir()?;
    let abi_root = generate_abi_fn_with! {
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
    assert_eq!(abi_root, serde_json::from_slice(&abi_json)?);

    Ok(())
}

#[test]
#[named]
fn test_abi_opt_features() -> testresult::TestResult {
    setup_tracing();
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/abi/_Cargo_features.toml";
        Opts: "--features gated";
        Code:
        #[cfg(feature = "gated")]
        pub fn gated_only(&self) -> bool {
            true
        }
    };

    let function_names: Vec<&str> = abi_root
        .body
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect();

    assert!(
        function_names.contains(&"gated_only"),
        "expected the ABI to include `gated_only` when the feature is enabled"
    );

    Ok(())
}
