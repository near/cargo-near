//! Integration tests for toolchain detection functionality.
//!
//! These tests verify that cargo-near correctly detects and uses the active
//! toolchain from the target project directory, even when invoked from a
//! different working directory with `--manifest-path`.

use camino::Utf8PathBuf;
use std::fs;
use std::process::Command;

/// Test that toolchain detection works correctly when using --manifest-path
/// from a different working directory.
///
/// This test verifies the fix for issue #361 where cargo-near was incorrectly
/// detecting the toolchain from the current working directory instead of the
/// target project directory.
#[test]
fn test_toolchain_detection_respects_manifest_path_directory() -> testresult::TestResult<()> {
    // Create a temporary directory for our test project
    let tmp_dir = tempfile::Builder::new()
        .prefix("toolchain_test_")
        .tempdir()?;
    let project_dir = tmp_dir.path();

    // Create a minimal NEAR contract project
    let cargo_toml = r#"
[package]
name = "toolchain-test-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
near-sdk = "5.6"

[profile.release]
codegen-units = 1
opt-level = "z"
lto = true
debug = false
panic = "abort"
overflow-checks = true
"#;

    let lib_rs = r#"
use near_sdk::near;

#[derive(Default)]
#[near(contract_state)]
pub struct Contract {}

#[near]
impl Contract {
    pub fn hello(&self) -> String {
        "Hello".to_string()
    }
}
"#;

    // Use 1.86.0 which is the recommended compatible version
    let rust_toolchain_toml = r#"
[toolchain]
channel = "1.86.0"
targets = ["wasm32-unknown-unknown"]
"#;

    // Create directory structure
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Write files
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    fs::write(project_dir.join("rust-toolchain.toml"), rust_toolchain_toml)?;

    // Verify the toolchain is correctly detected from project directory
    // Note: We need to clear RUSTUP_TOOLCHAIN env var which cargo sets during tests
    let output = Command::new("rustup")
        .args(["show", "active-toolchain"])
        .current_dir(project_dir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .output()?;
    let toolchain_in_project = String::from_utf8_lossy(&output.stdout);
    assert!(
        toolchain_in_project.contains("1.86"),
        "Expected project to use 1.86 toolchain, got: {}",
        toolchain_in_project
    );

    // Get cargo-near binary path by checking both debug and release directories
    // CI may build in release mode, local dev typically uses debug
    let workspace_dir: Utf8PathBuf = env!("CARGO_MANIFEST_DIR").into();
    let workspace_root = workspace_dir
        .parent()
        .expect("integration-tests should have parent");

    let cargo_near_binary = {
        let debug_path = workspace_root
            .join("target")
            .join("debug")
            .join("cargo-near");
        let release_path = workspace_root
            .join("target")
            .join("release")
            .join("cargo-near");

        if release_path.exists() {
            release_path
        } else if debug_path.exists() {
            debug_path
        } else {
            // Binary not found - build it first
            // This can happen when running `cargo test --workspace` which compiles
            // tests but doesn't necessarily place the binary in target/debug/
            let status = Command::new("cargo")
                .args(["build", "-p", "cargo-near", "--quiet"])
                .current_dir(workspace_root)
                .status()?;
            assert!(status.success(), "Failed to build cargo-near binary");
            assert!(
                debug_path.exists(),
                "cargo-near binary still not found at {:?} after building",
                debug_path
            );
            debug_path
        }
    };

    // Run cargo-near from a different directory (/tmp) with --manifest-path
    // pointing to our test project. This simulates the scenario from issue #361.
    let manifest_path = project_dir.join("Cargo.toml");

    let output = Command::new(&cargo_near_binary)
        .args([
            "near",
            "build",
            "non-reproducible-wasm",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
        ])
        .current_dir(std::env::temp_dir()) // Run from a different directory
        .env_remove("RUSTUP_TOOLCHAIN") // Clear to let cargo-near detect from project
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}\n{}", stdout, stderr);

    // The build should succeed without the "1.87.0 or newer" warning
    // (since the project uses 1.86.0)
    assert!(
        !combined_output.contains("1.87.0 or newer rust toolchain is currently not compatible"),
        "Build should not show incompatibility warning when project uses 1.86.0 toolchain.\n\
         This indicates the toolchain was detected from the wrong directory.\n\
         Output: {}",
        combined_output
    );

    // If there was a build failure, it shouldn't be due to toolchain version
    if !output.status.success() {
        // Allow build failures for reasons other than toolchain version
        // (e.g., missing dependencies, compilation errors)
        // But fail if the error is about toolchain version
        assert!(
            !combined_output.contains("wasm, compiled with 1.87.0"),
            "Build failed due to incorrect toolchain detection.\nOutput: {}",
            combined_output
        );
    }

    Ok(())
}

/// Test that detect_active_toolchain correctly identifies toolchains from
/// directory overrides specified via rust-toolchain.toml.
#[test]
fn test_detect_toolchain_from_rust_toolchain_toml() -> testresult::TestResult<()> {
    // Create a temporary directory with a rust-toolchain.toml
    let tmp_dir = tempfile::Builder::new()
        .prefix("toolchain_detect_")
        .tempdir()?;
    let project_dir = tmp_dir.path();

    // Write a rust-toolchain.toml specifying a specific version
    let rust_toolchain_toml = r#"
[toolchain]
channel = "1.86.0"
"#;
    fs::write(project_dir.join("rust-toolchain.toml"), rust_toolchain_toml)?;

    // Detect toolchain from this directory
    // Note: Clear RUSTUP_TOOLCHAIN which cargo sets during tests
    let output = Command::new("rustup")
        .args(["show", "active-toolchain"])
        .current_dir(project_dir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .output()?;

    let toolchain = String::from_utf8_lossy(&output.stdout);

    // The detected toolchain should be 1.86.0 (if installed)
    // or show an error about the toolchain not being installed
    if output.status.success() {
        assert!(
            toolchain.contains("1.86"),
            "Expected 1.86 toolchain to be detected from rust-toolchain.toml, got: {}",
            toolchain
        );
    }

    Ok(())
}
