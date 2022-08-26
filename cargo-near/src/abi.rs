use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::{util, AbiCommand};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const ABI_FILE: &str = "abi.json";

/// ABI generation result.
pub(crate) struct AbiResult {
    /// Path to the resulting ABI file.
    pub path: PathBuf,
}

pub(crate) fn write_to_file(
    crate_metadata: &CrateMetadata,
    generate_docs: bool,
) -> anyhow::Result<AbiResult> {
    let original_cargo_toml: toml::value::Table =
        toml::from_slice(&fs::read(&crate_metadata.manifest_path.path)?)?;

    if !original_cargo_toml["dependencies"]["near-sdk"]["features"]
        .as_array()
        .map_or(false, |features| {
            features.contains(&toml::Value::String("abi".to_string()))
        })
    {
        anyhow::bail!("Unable to generate ABI: NEAR SDK \"abi\" feature is not enabled")
    }

    let dylib_artifact = util::compile_project(
        &crate_metadata.manifest_path,
        &["--features", "near-sdk/__abi-generate"],
        vec![
            ("CARGO_PROFILE_DEV_OPT_LEVEL", "0"),
            ("CARGO_PROFILE_DEV_DEBUG", "0"),
            ("CARGO_PROFILE_DEV_LTO", "off"),
        ],
        util::dylib_extension(),
    )?;

    let abi_entries = util::extract_abi_entries(&dylib_artifact.path)?;

    let mut contract_abi = near_abi::__private::ChunkedAbiEntry::combine(abi_entries)?
        .into_abi_root(extract_metadata(crate_metadata));

    if !generate_docs {
        strip_docs(&mut contract_abi);
    }
    let near_abi_json = serde_json::to_string(&contract_abi)?;
    let out_path_abi = crate_metadata.target_directory.join(ABI_FILE);
    fs::write(&out_path_abi, near_abi_json)?;

    Ok(AbiResult { path: out_path_abi })
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

fn strip_docs(abi_root: &mut near_abi::AbiRoot) {
    for function in &mut abi_root.body.functions {
        function.doc = None;
    }
    for schema in &mut abi_root.body.root_schema.definitions.values_mut() {
        if let schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            metadata: Some(metadata),
            ..
        }) = schema
        {
            metadata.description = None;
        }
    }
}

pub(crate) fn run(args: AbiCommand) -> anyhow::Result<()> {
    let crate_metadata = CrateMetadata::collect(CargoManifestPath::try_from(
        args.manifest_path.unwrap_or_else(|| "Cargo.toml".into()),
    )?)?;

    let AbiResult { path } = write_to_file(&crate_metadata, args.doc)?;

    println!("ABI successfully generated at {}", path.display());

    Ok(())
}
