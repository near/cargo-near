use std::collections::HashMap;
use std::fs;

use camino::Utf8PathBuf;
use cargo_near_build::cargo_native::{self, DYLIB};
use cargo_near_build::near::abi;
use cargo_near_build::pretty_print;
use cargo_near_build::types::cargo::manifest_path::ManifestPath;
use color_eyre::eyre::ContextCompat;
use colored::Colorize;
use near_abi::AbiRoot;

use crate::commands::build_command::BUILD_RS_ABI_STEP_HINT_ENV_KEY;
use crate::types::metadata::CrateMetadata;
use cargo_near_build::types::color_preference::ColorPreference;

/// ABI generation result.
pub(crate) struct AbiResult {
    /// Path to the resulting ABI file.
    pub path: Utf8PathBuf,
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
    no_locked: bool,
    generate_docs: bool,
    hide_warnings: bool,
    cargo_feature_args: &[&str],
    color: ColorPreference,
) -> color_eyre::eyre::Result<AbiRoot> {
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
        .wrap_err("unable to appropriately resolve the dependency graph, perhaps your `Cargo.toml` file is malformed")?;

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
        .wrap_err("`near-sdk` dependency not found")?;

    for required_feature in ["__abi-generate", "__abi-embed"] {
        if !near_sdk_dep.features.contains_key(required_feature) {
            color_eyre::eyre::bail!(
                "{}: {}",
                format!(
                    "missing `{}` required feature for `near-sdk` dependency",
                    required_feature
                ),
                "probably unsupported `near-sdk` version. expected 4.1.* or higher"
            );
        }
    }

    let cargo_args = {
        let mut args = vec!["--features", "near-sdk/__abi-generate"];
        args.extend_from_slice(cargo_feature_args);
        if !no_locked {
            args.push("--locked");
        }

        args
    };

    pretty_print::step("Generating ABI");

    let dylib_artifact = cargo_native::compile::run::<DYLIB>(
        &crate_metadata.manifest_path,
        cargo_args.as_slice(),
        vec![
            ("CARGO_PROFILE_DEV_OPT_LEVEL", "0"),
            ("CARGO_PROFILE_DEV_DEBUG", "0"),
            ("CARGO_PROFILE_DEV_LTO", "off"),
            (BUILD_RS_ABI_STEP_HINT_ENV_KEY, "true"),
        ],
        hide_warnings,
        color,
    )?;

    let mut contract_abi = pretty_print::handle_step("Extracting ABI...", || {
        let abi_entries = abi::dylib::extract_abi_entries(&dylib_artifact)?;
        Ok(near_abi::__private::ChunkedAbiEntry::combine(abi_entries)?
            .into_abi_root(extract_metadata(crate_metadata)))
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
) -> color_eyre::eyre::Result<AbiResult> {
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
        crate_metadata.formatted_package_name(),
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
pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    pub no_locked: bool,
    /// Include rustdocs in the ABI file
    pub no_doc: bool,
    /// Generate compact (minified) JSON
    pub compact_abi: bool,
    /// Copy final artifacts to this directory
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Coloring: auto, always, never
    pub color: Option<cargo_near_build::types::color_preference::ColorPreference>,
}

impl From<super::AbiCommand> for Opts {
    fn from(value: super::AbiCommand) -> Self {
        Self {
            no_locked: value.no_locked,
            no_doc: value.no_doc,
            compact_abi: value.compact_abi,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            color: value.color.map(Into::into),
        }
    }
}

pub fn run(args: Opts) -> near_cli_rs::CliResult {
    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        let manifest_path: Utf8PathBuf = if let Some(manifest_path) = args.manifest_path {
            manifest_path.into()
        } else {
            "Cargo.toml".into()
        };
        CrateMetadata::collect(ManifestPath::try_from(manifest_path)?, args.no_locked)
    })?;

    let out_dir = crate_metadata.resolve_output_dir(args.out_dir)?;

    let format = if args.compact_abi {
        AbiFormat::JsonMin
    } else {
        AbiFormat::Json
    };
    let contract_abi = generate_abi(
        &crate_metadata,
        args.no_locked,
        !args.no_doc,
        false,
        &[],
        color,
    )?;
    let AbiResult { path } =
        write_to_file(&contract_abi, &crate_metadata, format, AbiCompression::NoOp)?;

    let abi_path = cargo_near_build::fs::copy(&path, &out_dir)?;

    pretty_print::success("ABI Successfully Generated!");
    eprintln!("     - ABI: {}", abi_path.to_string().yellow().bold());

    Ok(())
}
