use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::{util, AbiCommand};
use colored::Colorize;
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
    hide_warnings: bool,
) -> anyhow::Result<AbiRoot> {
    let root_node = crate_metadata
        .raw_metadata
        .resolve
        .as_ref()
        .and_then(|dep_graph| {
            dep_graph
            .nodes
            .iter()
            .find(|node| node.id == crate_metadata.root_package.id)
        })
        .ok_or_else(|| anyhow::anyhow!("unable to appropriately resolve the dependency graph, perhaps your `Cargo.toml` file is malformed"))?;

    let near_sdk_dep = root_node
        .deps
        .iter()
        .find(|dep| dep.name == "near_sdk")
        .and_then(|near_sdk| {
            crate_metadata
                .raw_metadata
                .packages
                .iter()
                .find(|pkg| pkg.id == near_sdk.pkg)
        })
        .ok_or_else(|| anyhow::anyhow!("`near-sdk` dependency not found"))?;

    for required_feature in ["__abi-generate", "__abi-embed"] {
        if !near_sdk_dep.features.contains_key(required_feature) {
            anyhow::bail!("unsupported `near-sdk` version. expected 4.1.* or higher");
        }
    }

    if !crate_metadata
        .root_package
        .dependencies
        .iter()
        .find(|dep| dep.name == "near-sdk")
        .ok_or_else(|| anyhow::anyhow!("`near-sdk` dependency not found"))?
        .features
        .iter()
        .any(|feature| feature == "abi")
    {
        anyhow::bail!("`near-sdk` dependency must have the `abi` feature enabled")
    }

    util::print_step("Generating ABI");
    let dylib_artifact = util::compile_project(
        &crate_metadata.manifest_path,
        &["--features", "near-sdk/__abi-generate"],
        vec![
            ("CARGO_PROFILE_DEV_OPT_LEVEL", "0"),
            ("CARGO_PROFILE_DEV_DEBUG", "0"),
            ("CARGO_PROFILE_DEV_LTO", "off"),
        ],
        util::dylib_extension(),
        hide_warnings,
    )?;

    let mut contract_abi = util::handle_step("Extracting ABI...", || {
        let abi_entries = util::extract_abi_entries(&dylib_artifact.path)?;
        anyhow::Ok(
            near_abi::__private::ChunkedAbiEntry::combine(abi_entries)?
                .into_abi_root(extract_metadata(crate_metadata)),
        )
    })?;

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

    fs::create_dir_all(&crate_metadata.target_directory)?;

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
        build: None,
        wasm_hash: None,
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
    let crate_metadata = util::handle_step("Collecting cargo project metadata...", || {
        CrateMetadata::collect(CargoManifestPath::try_from(
            args.manifest_path.unwrap_or_else(|| "Cargo.toml".into()),
        )?)
    })?;

    let out_dir = util::force_canonicalize_dir(
        &args
            .out_dir
            .unwrap_or_else(|| crate_metadata.target_directory.clone()),
    )?;

    let format = if args.compact_abi {
        AbiFormat::JsonMin
    } else {
        AbiFormat::Json
    };
    let contract_abi = generate_abi(&crate_metadata, args.doc, false)?;
    let AbiResult { path } =
        write_to_file(&contract_abi, &crate_metadata, format, AbiCompression::NoOp)?;

    let abi_path = util::copy(&path, &out_dir)?;

    util::print_success("ABI Successfully Generated!");
    eprintln!(
        "     - ABI: {}",
        abi_path.display().to_string().yellow().bold()
    );

    Ok(())
}
