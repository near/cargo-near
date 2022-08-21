use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::util;

use super::AbiCommand;

mod generation;

const ABI_FILE: &str = "abi.json";

pub(crate) struct AbiResult {
    pub source_hash: u64,
    pub path: PathBuf,
}

pub(crate) fn write_to_file(crate_metadata: &CrateMetadata) -> anyhow::Result<AbiResult> {
    let near_abi_gen_dir = &crate_metadata
        .target_directory
        .join(crate_metadata.root_package.name.clone() + "-near-abi-gen");
    fs::create_dir_all(near_abi_gen_dir)?;
    log::debug!(
        "Using temp Cargo workspace at '{}'",
        near_abi_gen_dir.display()
    );

    let dylib_artifact = util::compile_project(
        &crate_metadata.manifest_path,
        &[],
        vec![
            ("CARGO_PROFILE_DEV_OPT_LEVEL", "0"),
            ("CARGO_PROFILE_DEV_DEBUG", "0"),
            ("CARGO_PROFILE_DEV_LTO", "off"),
        ],
        util::dylib_extension(),
    )?;

    // todo! re-use cargo-near for extracting data from the dylib
    // todo! instead of a temp project
    if dylib_artifact.fresh {
        let cargo_toml = generation::generate_toml(&crate_metadata.manifest_path)?;
        fs::write(near_abi_gen_dir.join("Cargo.toml"), cargo_toml)?;

        let build_rs = generation::generate_build_rs(&dylib_artifact.path)?;
        fs::write(near_abi_gen_dir.join("build.rs"), build_rs)?;

        let main_rs = generation::generate_main_rs(&dylib_artifact.path)?;
        fs::write(near_abi_gen_dir.join("main.rs"), main_rs)?;
    }

    let stdout = util::invoke_cargo(
        "run",
        &["--package", "near-abi-gen"],
        Some(near_abi_gen_dir),
        vec![(
            "LD_LIBRARY_PATH",
            &dylib_artifact.path.parent().unwrap().to_string_lossy(),
        )],
    )?;

    let combined_entries =
        near_abi::ChunkedAbiEntry::combine(serde_json::from_slice::<Vec<_>>(&stdout)?.into_iter())?;

    let source_hash = combined_entries.source_hash;

    let contract_abi = near_abi::AbiRoot::new(extract_metadata(crate_metadata), combined_entries);

    let near_abi_json = serde_json::to_string(&contract_abi)?;
    let out_path_abi = crate_metadata.target_directory.join(ABI_FILE);
    fs::write(&out_path_abi, near_abi_json)?;

    Ok(AbiResult {
        source_hash,
        path: out_path_abi,
    })
}

fn extract_metadata(crate_metadata: &CrateMetadata) -> near_abi::AbiMetadata {
    let package = &crate_metadata.root_package;
    near_abi::AbiMetadata {
        name: Some(package.name.clone()),
        version: Some(package.version.to_string()),
        authors: package.authors.clone(),
        other: HashMap::new(),
    }
}

pub(crate) fn dump_to_file(args: AbiCommand) -> anyhow::Result<()> {
    let crate_metadata = CrateMetadata::collect(CargoManifestPath::try_from(
        args.manifest_path.unwrap_or_else(|| "Cargo.toml".into()),
    )?)?;

    let AbiResult { path, .. } = write_to_file(&crate_metadata)?;

    println!("ABI successfully generated at {}", path.display());

    Ok(())
}
