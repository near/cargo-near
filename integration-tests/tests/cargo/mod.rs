use cargo_near_integration_tests::{generate_abi_fn_with, generate_abi_with, SDK_GIT_REV};
use function_name::named;
use git2::build::CheckoutBuilder;
use git2::Repository;
use std::collections::HashMap;
use tempfile::TempDir;

use crate::util::AsJsonSchema;

fn clone_git_repo() -> anyhow::Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let repo_dir = temp_dir.path();
    let repo = Repository::clone("https://github.com/near/near-sdk-rs", &repo_dir)?;
    let commit = repo.revparse_single(SDK_GIT_REV)?;
    repo.checkout_tree(&commit, Some(&mut CheckoutBuilder::new()))?;

    Ok(temp_dir)
}

#[test]
#[named]
fn test_dependency_local_path() -> anyhow::Result<()> {
    let near_sdk_dir = clone_git_repo()?;
    let near_sdk_dep_path = near_sdk_dir.path().join("near-sdk");

    // near-sdk = { path = "::path::", features = ["abi"] }
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_local_path.toml";
        Vars: HashMap::from([("path", near_sdk_dep_path.to_str().unwrap())]);
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_local_path_with_version() -> anyhow::Result<()> {
    let near_sdk_dir = clone_git_repo()?;
    let near_sdk_dep_path = near_sdk_dir.path().join("near-sdk");

    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_local_path_with_version.toml";
        Vars: HashMap::from([("path", near_sdk_dep_path.to_str().unwrap())]);
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_explicit() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_explicit.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_no_default_features() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_no_default_features.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_multiple_features() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_multiple_features.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_platform_specific() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_platform_specific.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

// Does not work because of NEAR SDK (generates code that depends on `near-sdk` being the package name).
#[ignore]
#[test]
#[named]
fn test_dependency_renamed() -> anyhow::Result<()> {
    let abi_root = generate_abi_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_renamed.toml";
        Code:
        use near::borsh::{self, BorshDeserialize, BorshSerialize};
        use near::near_bindgen;

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
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

// TODO: add support for patch section
#[ignore]
#[test]
#[named]
fn test_dependency_patch() -> anyhow::Result<()> {
    // [dependencies]
    // near-sdk = "4.0.0"
    //
    // [patch.crates-io]
    // near-sdk = { git = "https://github.com/near/near-sdk-rs.git", rev = "91a44a621732e92723dfb58c377bb2135959ad8f" }
    let abi_root = generate_abi_fn_with! {
        Cargo: "/templates/sdk-dependency/_Cargo_patch.toml";
        Code:
        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);

    Ok(())
}

// TODO: Re-enable when we release 4.1.0
#[ignore]
#[test]
#[named]
fn test_abi_not_a_table() -> anyhow::Result<()> {
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
