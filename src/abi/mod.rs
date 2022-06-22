use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::util;
use near_sdk::__private::{AbiMetadata, AbiRoot};
use std::collections::HashMap;
use std::{fs, path::PathBuf};

mod generation;

const ABI_FILE: &str = "abi.json";

/// ABI generation result.
pub(crate) struct AbiResult {
    /// Path to the resulting ABI file.
    pub path: PathBuf,
}

pub(crate) fn execute(manifest_path: &CargoManifestPath) -> anyhow::Result<AbiResult> {
    let tmp_dir = tempfile::Builder::new()
        .prefix("cargo-contract_")
        .tempdir()?;
    log::debug!(
        "Using temp Cargo workspace at '{}'",
        tmp_dir.path().display()
    );

    let crate_metadata = CrateMetadata::collect(manifest_path)?;
    let dylib_path = util::compile_dylib_project(manifest_path)?;

    let cargo_toml = generation::generate_toml(manifest_path)?;
    fs::write(tmp_dir.path().join("Cargo.toml"), cargo_toml)?;

    let build_rs = generation::generate_build_rs(&dylib_path)?;
    fs::write(tmp_dir.path().join("build.rs"), build_rs)?;

    let main_rs = generation::generate_main_rs(&dylib_path)?;
    fs::write(tmp_dir.path().join("main.rs"), main_rs)?;

    let target_dir_arg = format!(
        "--target-dir={}",
        crate_metadata.target_directory.to_string_lossy()
    );
    let stdout = util::invoke_cargo(
        "run",
        &["--package", "near-abi-gen", "--release", &target_dir_arg],
        Some(tmp_dir.path()),
        vec![(
            "LD_LIBRARY_PATH",
            &dylib_path.parent().unwrap().to_string_lossy(),
        )],
    )?;

    let mut near_abi: AbiRoot = serde_json::from_slice(&stdout)?;
    let metadata = extract_metadata(&crate_metadata);
    near_abi.metadata = metadata;
    let near_abi_json = serde_json::to_string_pretty(&near_abi)?;
    let out_path_abi = crate_metadata.target_directory.join(ABI_FILE);
    fs::write(&out_path_abi, near_abi_json)?;

    Ok(AbiResult { path: out_path_abi })
}

fn extract_metadata(crate_metadata: &CrateMetadata) -> AbiMetadata {
    let package = &crate_metadata.root_package;
    AbiMetadata {
        name: Some(package.name.clone()),
        version: Some(package.version.to_string()),
        authors: package.authors.clone(),
        other: HashMap::new(),
    }
}
