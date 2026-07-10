//! Exercises `cargo_near_build::list::list_contracts` — the discovery primitive behind
//! `cargo near build list` — against a synthesized multi-crate workspace. Only `cargo metadata`
//! runs here; nothing is compiled, so the test is fast and offline.

use camino::Utf8PathBuf;
use std::fs;
use std::path::Path;

fn write(path: &Path, contents: &str) -> testresult::TestResult {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

/// Lays out a workspace with two contracts (one with named variants, one without) plus a plain
/// library crate that opts out of reproducible builds. Members are listed out of alphabetical
/// order on purpose, to check that discovery sorts them.
fn scaffold_workspace(root: &Path) -> testresult::TestResult {
    write(
        &root.join("Cargo.toml"),
        r#"[workspace]
resolver = "2"
members = ["contract-b", "contract-a", "plain-lib"]
"#,
    )?;

    // contract-a: reproducible build, default variant only.
    write(
        &root.join("contract-a/Cargo.toml"),
        r#"[package]
name = "contract-a"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[package.metadata.near.reproducible_build]
image = "sourcescan/cargo-near:0.13.0-rust-1.84.0"
image_digest = "sha256:deadbeef"
"#,
    )?;
    write(&root.join("contract-a/src/lib.rs"), "")?;

    // contract-b: reproducible build with two named variants, declared out of order.
    write(
        &root.join("contract-b/Cargo.toml"),
        r#"[package]
name = "contract-b"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[package.metadata.near.reproducible_build]
image = "sourcescan/cargo-near:0.13.0-rust-1.84.0"
image_digest = "sha256:deadbeef"

[package.metadata.near.reproducible_build.variant.zebra]
image = "sourcescan/cargo-near:0.13.0-rust-1.84.0"
image_digest = "sha256:zebra"

[package.metadata.near.reproducible_build.variant.alpha]
image = "sourcescan/cargo-near:0.13.0-rust-1.84.0"
image_digest = "sha256:alpha"
"#,
    )?;
    write(&root.join("contract-b/src/lib.rs"), "")?;

    // plain-lib: no reproducible_build section, so it must be filtered out.
    write(
        &root.join("plain-lib/Cargo.toml"),
        r#"[package]
name = "plain-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]
"#,
    )?;
    write(&root.join("plain-lib/src/lib.rs"), "")?;

    Ok(())
}

#[test]
fn list_contracts_discovers_variants_and_filters_non_contracts() -> testresult::TestResult {
    let tmp = tempfile::tempdir()?;
    scaffold_workspace(tmp.path())?;
    let manifest = Utf8PathBuf::from_path_buf(tmp.path().join("Cargo.toml"))
        .expect("temp path is valid UTF-8");

    let workspace = cargo_near_build::list::list_contracts(Some(manifest.as_path()))?;
    let contracts = &workspace.contracts;

    // plain-lib is excluded; the two contracts come back sorted by package name.
    let names: Vec<&str> = contracts.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["contract-a", "contract-b"]);

    // Manifest paths are relative to the workspace root, and resolve back to real files.
    assert_eq!(contracts[0].manifest_path, "contract-a/Cargo.toml");
    assert_eq!(contracts[1].manifest_path, "contract-b/Cargo.toml");
    assert!(workspace.root.join(&contracts[0].manifest_path).is_file());

    // contract-a: default variant only.
    assert_eq!(contracts[0].variants, vec![None]);

    // contract-b: default first, then named variants sorted (alpha before zebra).
    assert_eq!(
        contracts[1].variants,
        vec![None, Some("alpha".to_string()), Some("zebra".to_string())]
    );

    // Enumeration flattens to one build unit per (contract, variant).
    let units: Vec<_> = contracts.iter().flat_map(|c| c.build_units()).collect();
    let rows: Vec<(&str, Option<&str>, &str)> = units
        .iter()
        .map(|u| (u.package.as_str(), u.variant.as_deref(), u.output.as_str()))
        .collect();
    // Output filenames match cargo-near's artifact naming (hyphens become underscores) and do not
    // depend on the variant, so contract-b's three units all write `contract_b.wasm`.
    assert_eq!(
        rows,
        [
            ("contract-a", None, "contract_a.wasm"),
            ("contract-b", None, "contract_b.wasm"),
            ("contract-b", Some("alpha"), "contract_b.wasm"),
            ("contract-b", Some("zebra"), "contract_b.wasm"),
        ]
    );

    Ok(())
}
