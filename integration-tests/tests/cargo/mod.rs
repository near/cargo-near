use cargo_near_integration_tests::{generate_abi, generate_abi_fn};
use function_name::named;
use git2::build::CheckoutBuilder;
use git2::Repository;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn clone_git_repo(version: &str) -> anyhow::Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let repo_dir = temp_dir.path();
    let repo = Repository::clone("https://github.com/near/near-sdk-rs", &repo_dir)?;
    let commit = repo.revparse_single(version)?;
    repo.checkout_tree(&commit, Some(&mut CheckoutBuilder::new()))?;

    Ok(temp_dir)
}

#[test]
#[named]
fn test_dependency_local_path() -> anyhow::Result<()> {
    let near_sdk_dir = clone_git_repo("83cb1c40403e4156bd6d1d258237bbdf03326afc")?;
    let near_sdk_dep_path = near_sdk_dir.path().join("near-sdk");

    // near-sdk = { path = "::path::", features = ["abi"] }
    let abi_root = generate_abi_fn! {
        with Cargo "/templates/sdk-dependency/_Cargo_local_path.toml",
        and vars HashMap::from([("path", near_sdk_dep_path.to_str().unwrap())]);

        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_local_path_with_version() -> anyhow::Result<()> {
    let near_sdk_dir = clone_git_repo("83cb1c40403e4156bd6d1d258237bbdf03326afc")?;
    let near_sdk_dep_path = near_sdk_dir.path().join("near-sdk");

    // near-sdk = { path = "::path::", version = "4.1.0-pre.2", features = ["abi"] }
    let abi_root = generate_abi_fn! {
        with Cargo "/templates/sdk-dependency/_Cargo_local_path_with_version.toml",
        and vars HashMap::from([("path", near_sdk_dep_path.to_str().unwrap())]);

        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_explicit() -> anyhow::Result<()> {
    // [dependencies.near-sdk]
    // version = "4.1.0-pre.2"
    // features = ["abi"]
    let abi_root = generate_abi_fn! {
        with Cargo "/templates/sdk-dependency/_Cargo_explicit.toml";

        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_no_default_features() -> anyhow::Result<()> {
    // near-sdk = { version = "4.1.0-pre.2", default-features = false, features = ["abi"] }
    let abi_root = generate_abi_fn! {
        with Cargo "/templates/sdk-dependency/_Cargo_no_default_features.toml";

        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.params.len(), 2);

    Ok(())
}

#[test]
#[named]
fn test_dependency_multiple_features() -> anyhow::Result<()> {
    // near-sdk = { version = "4.1.0-pre.2", features = ["abi", "unstable"] }
    let abi_root = generate_abi_fn! {
        with Cargo "/templates/sdk-dependency/_Cargo_multiple_features.toml";

        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.params.len(), 2);

    Ok(())
}

// TODO: Not sure if this even makes sense in the context of NEAR SDK, but in theory could add support for this
#[ignore]
#[test]
#[named]
fn test_dependency_platform_specific() -> anyhow::Result<()> {
    // [target.'cfg(windows)'.dependencies]
    // near-sdk = { version = "4.1.0-pre.2", features = ["abi"] }
    //
    // [target.'cfg(unix)'.dependencies]
    // near-sdk = { version = "4.1.0-pre.2", features = ["abi"] }
    let abi_root = generate_abi_fn! {
        with Cargo "/templates/sdk-dependency/_Cargo_platform_specific.toml";

        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.params.len(), 2);

    Ok(())
}

// Does not work because of NEAR SDK (generates code that depends on `near-sdk` being the package name).
#[ignore]
#[test]
#[named]
fn test_dependency_renamed() -> anyhow::Result<()> {
    // near = { version = "4.1.0-pre.2", package = "near-sdk", features = ["abi"] }
    let abi_root = generate_abi! {
        with Cargo "/templates/sdk-dependency/_Cargo_renamed.toml";

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
    assert_eq!(function.params.len(), 2);

    Ok(())
}

// TODO: add support for patch section
#[ignore]
#[test]
#[named]
fn test_dependency_patch() -> anyhow::Result<()> {
    // [dependencies]
    // near-sdk = { version = "4.1.0-pre.2", features = ["abi"] }
    //
    // [patch.crates-io]
    // near-sdk = { git = "https://github.com/near/near-sdk-rs.git", rev = "83cb1c40403e4156bd6d1d258237bbdf03326afc" }
    let abi_root = generate_abi_fn! {
        with Cargo "/templates/sdk-dependency/_Cargo_patch.toml";

        pub fn foo(&self, a: bool, b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.params.len(), 2);

    Ok(())
}
