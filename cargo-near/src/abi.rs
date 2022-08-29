use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::{util, AbiCommand};
use near_abi::AbiRoot;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// ABI generation result.
pub(crate) struct AbiResult {
    /// Path to the resulting ABI file.
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) enum AbiFormat {
    Json,
    JsonMin,
}

#[derive(Clone, Debug)]
pub(crate) enum AbiCompression {
    NoOp,
    Zstd,
}

pub(crate) fn generate_abi(
    crate_metadata: &CrateMetadata,
    generate_docs: bool,
) -> anyhow::Result<AbiRoot> {
    fs::create_dir_all(&crate_metadata.target_directory)?;

    let original_cargo_toml: toml::value::Table =
        toml::from_slice(&fs::read(&crate_metadata.manifest_path.path)?)?;

    if !original_cargo_toml
        .get("dependencies")
        .ok_or_else(|| anyhow::anyhow!("[dependencies] section not found"))?
        .get("near-sdk")
        .ok_or_else(|| anyhow::anyhow!("near-sdk dependency not found"))?
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("near-sdk dependency should be a table"))?
        .get("features")
        .and_then(|features| features.as_array())
        .map(|features| features.contains(&toml::Value::String("abi".to_string())))
        .unwrap_or(false)
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

    Ok(contract_abi)
}

pub(crate) fn write_to_file(
    contract_abi: &AbiRoot,
    crate_metadata: &CrateMetadata,
    format: AbiFormat,
    compression: AbiCompression,
) -> anyhow::Result<AbiResult> {
    let near_abi_serialized = match format {
        AbiFormat::Json => serde_json::to_vec_pretty(&contract_abi)?,
        AbiFormat::JsonMin => serde_json::to_vec(&contract_abi)?,
    };
    let near_abi_compressed = match compression {
        AbiCompression::NoOp => near_abi_serialized,
        AbiCompression::Zstd => zstd::encode_all(
            near_abi_serialized.as_slice(),
            *zstd::compression_level_range().end(),
        )?,
    };

    let out_path_abi = crate_metadata.target_directory.join(format!(
        "{}_abi.{}",
        crate_metadata.root_package.name.replace('-', "_"),
        abi_file_extension(format, compression)
    ));
    fs::write(&out_path_abi, near_abi_compressed)?;

    Ok(AbiResult { path: out_path_abi })
}

fn abi_file_extension(format: AbiFormat, compression: AbiCompression) -> &'static str {
    match compression {
        AbiCompression::NoOp => match format {
            AbiFormat::Json | AbiFormat::JsonMin => "json",
        },
        AbiCompression::Zstd => "zst",
    }
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

    let out_dir = util::force_canonicalize_dir(
        &args
            .out_dir
            .unwrap_or_else(|| crate_metadata.target_directory.clone()),
    )?;

    let format = if args.no_pretty {
        AbiFormat::JsonMin
    } else {
        AbiFormat::Json
    };
    let contract_abi = generate_abi(&crate_metadata, args.doc)?;
    let AbiResult { path } =
        write_to_file(&contract_abi, &crate_metadata, format, AbiCompression::NoOp)?;

    let abi_path = util::copy(&path, &out_dir)?;

    println!("ABI successfully generated at `{}`", abi_path.display());

    Ok(())
}
