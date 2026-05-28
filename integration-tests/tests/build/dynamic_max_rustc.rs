//! Integration tests for the dynamic max-rustc threshold driven by
//! `[package.metadata.near] min_protocol_version` on the resolved `near-sdk`.
//!
//! These tests exercise the metadata-reading helpers on `CrateMetadata` rather than the
//! wasm build path: a full build on rustc 1.93 would need a PV-84-ready `near-sdk`
//! published to crates.io, which doesn't exist yet.

use cargo_near_build::{CargoTargetDir, CrateMetadata};
use std::fs;

/// Sets up a tiny throwaway workspace at `<dir>` containing:
///   - a stub crate named `stub-near-sdk` whose Cargo.toml declares
///     `[package.metadata.near] min_protocol_version = <pv>` (if `pv` is Some),
///   - a contract crate that path-depends on the stub.
///
/// Returns the path to the contract's Cargo.toml.
fn build_fixture(dir: &std::path::Path, pv: Option<u32>) -> std::io::Result<camino::Utf8PathBuf> {
    let stub_dir = dir.join("stub-near-sdk");
    fs::create_dir_all(stub_dir.join("src"))?;
    let metadata_block = match pv {
        Some(pv) => format!("[package.metadata.near]\nmin_protocol_version = {pv}\n"),
        None => String::new(),
    };
    let stub_cargo_toml = format!(
        r#"[package]
name = "stub-near-sdk"
version = "0.0.1"
edition = "2021"

{metadata_block}
[lib]
path = "src/lib.rs"
"#
    );
    fs::write(stub_dir.join("Cargo.toml"), stub_cargo_toml)?;
    fs::write(stub_dir.join("src").join("lib.rs"), "")?;

    let contract_dir = dir.join("contract");
    fs::create_dir_all(contract_dir.join("src"))?;
    let contract_cargo_toml = r#"[package]
name = "fixture-contract"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
stub-near-sdk = { path = "../stub-near-sdk" }

[workspace]
members = []
"#;
    fs::write(contract_dir.join("Cargo.toml"), contract_cargo_toml)?;
    fs::write(
        contract_dir.join("src").join("lib.rs"),
        "#[unsafe(no_mangle)] pub extern \"C\" fn entry() {}\n",
    )?;

    Ok(camino::Utf8PathBuf::from_path_buf(contract_dir.join("Cargo.toml")).unwrap())
}

#[test]
fn test_find_package_in_graph_finds_stub_dep() -> testresult::TestResult {
    let tmp = tempfile::tempdir()?;
    let manifest = build_fixture(tmp.path(), Some(84))?;

    // `no_locked = true`: the fixture has no Cargo.lock for cargo metadata to require.
    let meta = CrateMetadata::collect(manifest.try_into()?, true, &CargoTargetDir::NoOp, None)?;

    let pkg = meta
        .find_package_in_graph("stub-near-sdk")
        .expect("stub-near-sdk should be present in the resolved graph");
    assert_eq!(pkg.name.as_str(), "stub-near-sdk");

    // A non-existent name returns None.
    assert!(
        meta.find_package_in_graph("totally-not-a-real-crate")
            .is_none()
    );

    Ok(())
}

/// The helper surfaces `min_protocol_version = 84` declared on a dep named `near-sdk`
/// (the name `near_sdk_min_protocol_version` hardcodes). The stub is a plain path dep
/// with no `package = "..."` rename, so its graph name is already `near-sdk`.
#[test]
fn test_near_sdk_min_protocol_version_reads_metadata_when_declared() -> testresult::TestResult {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path();

    let stub_dir = dir.join("near-sdk-stub");
    fs::create_dir_all(stub_dir.join("src"))?;
    fs::write(
        stub_dir.join("Cargo.toml"),
        r#"[package]
name = "near-sdk"
version = "0.0.1"
edition = "2021"

[package.metadata.near]
min_protocol_version = 84

[lib]
path = "src/lib.rs"
"#,
    )?;
    fs::write(stub_dir.join("src").join("lib.rs"), "")?;

    let contract_dir = dir.join("contract");
    fs::create_dir_all(contract_dir.join("src"))?;
    fs::write(
        contract_dir.join("Cargo.toml"),
        r#"[package]
name = "fixture-contract"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
near-sdk = { path = "../near-sdk-stub" }

[workspace]
members = []
"#,
    )?;
    fs::write(
        contract_dir.join("src").join("lib.rs"),
        "#[unsafe(no_mangle)] pub extern \"C\" fn entry() {}\n",
    )?;

    let manifest = camino::Utf8PathBuf::from_path_buf(contract_dir.join("Cargo.toml")).unwrap();
    // `no_locked = true`: the fixture has no Cargo.lock for cargo metadata to require.
    let meta = CrateMetadata::collect(manifest.try_into()?, true, &CargoTargetDir::NoOp, None)?;

    assert_eq!(meta.near_sdk_min_protocol_version(), Some(84));
    Ok(())
}

/// Back-compat path: a contract whose `near-sdk` dep doesn't declare the
/// metadata block returns None — which `cargo-near-build` interprets as
/// "fall through to the historical 1.86 floor".
#[test]
fn test_near_sdk_min_protocol_version_none_when_metadata_absent() -> testresult::TestResult {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path();

    let stub_dir = dir.join("near-sdk-stub");
    fs::create_dir_all(stub_dir.join("src"))?;
    fs::write(
        stub_dir.join("Cargo.toml"),
        r#"[package]
name = "near-sdk"
version = "0.0.1"
edition = "2021"

[lib]
path = "src/lib.rs"
"#,
    )?;
    fs::write(stub_dir.join("src").join("lib.rs"), "")?;

    let contract_dir = dir.join("contract");
    fs::create_dir_all(contract_dir.join("src"))?;
    fs::write(
        contract_dir.join("Cargo.toml"),
        r#"[package]
name = "fixture-contract"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
near-sdk = { path = "../near-sdk-stub" }

[workspace]
members = []
"#,
    )?;
    fs::write(
        contract_dir.join("src").join("lib.rs"),
        "#[unsafe(no_mangle)] pub extern \"C\" fn entry() {}\n",
    )?;

    let manifest = camino::Utf8PathBuf::from_path_buf(contract_dir.join("Cargo.toml")).unwrap();
    // `no_locked = true`: the fixture has no Cargo.lock for cargo metadata to require.
    let meta = CrateMetadata::collect(manifest.try_into()?, true, &CargoTargetDir::NoOp, None)?;

    assert_eq!(meta.near_sdk_min_protocol_version(), None);
    Ok(())
}

/// Transitive lookup: when `near-sdk` is pulled in via a re-exporter (here a
/// stand-in for `near-contract-standards`), `find_package_in_graph` should
/// still surface it from the full resolved graph.
#[test]
fn test_find_package_in_graph_finds_transitive_dep() -> testresult::TestResult {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path();

    let sdk_dir = dir.join("near-sdk-stub");
    fs::create_dir_all(sdk_dir.join("src"))?;
    fs::write(
        sdk_dir.join("Cargo.toml"),
        r#"[package]
name = "near-sdk"
version = "0.0.1"
edition = "2021"

[package.metadata.near]
min_protocol_version = 84

[lib]
path = "src/lib.rs"
"#,
    )?;
    fs::write(sdk_dir.join("src").join("lib.rs"), "")?;

    let standards_dir = dir.join("near-contract-standards-stub");
    fs::create_dir_all(standards_dir.join("src"))?;
    fs::write(
        standards_dir.join("Cargo.toml"),
        r#"[package]
name = "near-contract-standards"
version = "0.0.1"
edition = "2021"

[dependencies]
near-sdk = { path = "../near-sdk-stub" }

[lib]
path = "src/lib.rs"
"#,
    )?;
    fs::write(standards_dir.join("src").join("lib.rs"), "")?;

    let contract_dir = dir.join("contract");
    fs::create_dir_all(contract_dir.join("src"))?;
    // Contract depends only on near-contract-standards; near-sdk reaches it transitively.
    fs::write(
        contract_dir.join("Cargo.toml"),
        r#"[package]
name = "fixture-contract"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
near-contract-standards = { path = "../near-contract-standards-stub" }

[workspace]
members = []
"#,
    )?;
    fs::write(
        contract_dir.join("src").join("lib.rs"),
        "#[unsafe(no_mangle)] pub extern \"C\" fn entry() {}\n",
    )?;

    let manifest = camino::Utf8PathBuf::from_path_buf(contract_dir.join("Cargo.toml")).unwrap();
    // `no_locked = true`: the fixture has no Cargo.lock for cargo metadata to require.
    let meta = CrateMetadata::collect(manifest.try_into()?, true, &CargoTargetDir::NoOp, None)?;

    assert!(meta.find_package_in_graph("near-sdk").is_some());
    assert_eq!(meta.near_sdk_min_protocol_version(), Some(84));
    Ok(())
}

/// Reachability guard: a workspace sibling named `near-sdk` (declaring
/// `min_protocol_version = 84`) that the contract does NOT depend on must be ignored.
/// A flat name scan would false-positive; the graph-based lookup returns `None`.
#[test]
fn test_near_sdk_min_protocol_version_ignores_unrelated_sibling() -> testresult::TestResult {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path();

    let sibling_dir = dir.join("near-sdk");
    fs::create_dir_all(sibling_dir.join("src"))?;
    fs::write(
        sibling_dir.join("Cargo.toml"),
        r#"[package]
name = "near-sdk"
version = "0.0.1"
edition = "2021"

[package.metadata.near]
min_protocol_version = 84

[lib]
path = "src/lib.rs"
"#,
    )?;
    fs::write(sibling_dir.join("src").join("lib.rs"), "")?;

    // Contract does NOT depend on the sibling `near-sdk`.
    let contract_dir = dir.join("contract");
    fs::create_dir_all(contract_dir.join("src"))?;
    fs::write(
        contract_dir.join("Cargo.toml"),
        r#"[package]
name = "fixture-contract"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
"#,
    )?;
    fs::write(
        contract_dir.join("src").join("lib.rs"),
        "#[unsafe(no_mangle)] pub extern \"C\" fn entry() {}\n",
    )?;

    // Virtual workspace tying the two members together as siblings.
    fs::write(
        dir.join("Cargo.toml"),
        r#"[workspace]
resolver = "2"
members = ["near-sdk", "contract"]
"#,
    )?;

    let manifest = camino::Utf8PathBuf::from_path_buf(contract_dir.join("Cargo.toml")).unwrap();
    let meta = CrateMetadata::collect(manifest.try_into()?, true, &CargoTargetDir::NoOp, None)?;

    assert!(
        meta.find_package_in_graph("near-sdk").is_none(),
        "unrelated sibling near-sdk must not be found via the reachable graph"
    );
    assert_eq!(meta.near_sdk_min_protocol_version(), None);
    Ok(())
}
