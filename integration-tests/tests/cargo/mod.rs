use cargo_near_integration_tests::{from_git, generate_abi_fn_with, generate_abi_with};
use function_name::named;
use git2::build::CheckoutBuilder;
use git2::Repository;
use std::collections::HashMap;
use tempfile::TempDir;

use crate::util::AsJsonSchema;

fn clone_git_repo() -> testresult::TestResult<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let repo_dir = temp_dir.path();
    let repo = Repository::clone(from_git::SDK_REPO, repo_dir)?;
    let commit = repo.revparse_single(from_git::SDK_REVISION)?;
    repo.checkout_tree(&commit, Some(&mut CheckoutBuilder::new()))?;

    Ok(temp_dir)
}

#[test]
#[named]
fn test_dependency_local_path() -> testresult::TestResult {
    let near_sdk_dir = clone_git_repo()?;
    let near_sdk_dep_path = near_sdk_dir.path().join("near-sdk");

    // near-sdk = { path = "::path::", features = ["abi"] }
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_local_path.toml";
        Vars: HashMap::from([("path", near_sdk_dep_path.to_str().unwrap().to_owned())]);
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_local_path_with_version() -> testresult::TestResult {
    let near_sdk_dir = clone_git_repo()?;
    let near_sdk_dep_path = near_sdk_dir.path().join("near-sdk");

    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_local_path_with_version.toml";
        Vars: HashMap::from([("path", near_sdk_dep_path.to_str().unwrap().to_owned())]);
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_default_features() -> testresult::TestResult {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/_Cargo.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_explicit() -> testresult::TestResult {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_explicit.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_no_default_features() -> testresult::TestResult {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_no_default_features.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_multiple_features() -> testresult::TestResult {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_multiple_features.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_platform_specific() -> testresult::TestResult {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_platform_specific.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_renamed() -> testresult::TestResult {
    let abi_root = generate_abi_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_renamed.toml";
        Code:
        use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
        use near::near_bindgen;

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        #[borsh(crate = "near_sdk::borsh")]
        pub struct Contract {}

        #[near_bindgen]
        impl Contract {
            pub fn foo(&self, a: bool, b: u32) {}
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_patch() -> testresult::TestResult {
    // [dependencies]
    // near-sdk = "4.0.0"
    //
    // [patch.crates-io]
    // near-sdk = { git = "https://github.com/near/near-sdk-rs.git", rev = "10b0dea3b1a214d789cc90314aa814a4181610ad" }
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_patch.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

/// this is a test of Cargo.toml format
#[test]
#[named]
fn test_abi_not_a_table() -> testresult::TestResult {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_not_a_table.toml";
        Code:
        pub fn foo(&self, a: u32, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}
