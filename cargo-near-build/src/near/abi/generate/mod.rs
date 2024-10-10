use std::collections::HashMap;

use crate::types::near::build::input::ColorPreference;
use crate::{
    cargo_native::{self, Dylib},
    env_keys, pretty_print,
    types::cargo::metadata::CrateMetadata,
};
use eyre::ContextCompat;

pub mod dylib;

pub fn procedure(
    crate_metadata: &CrateMetadata,
    no_locked: bool,
    generate_docs: bool,
    hide_warnings: bool,
    cargo_feature_args: &[&str],
    env: &[(&str, &str)],
    color: ColorPreference,
) -> eyre::Result<near_abi::AbiRoot> {
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
            eyre::bail!(
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
        // TODO: add extenal no_release flag and react to it
        if false {
            args.push("--release");
        }

        args
    };

    pretty_print::step("Generating ABI");

    let compile_env = {
        let compile_env = vec![
            ("CARGO_PROFILE_DEV_OPT_LEVEL", "0"),
            ("CARGO_PROFILE_DEV_DEBUG", "0"),
            ("CARGO_PROFILE_DEV_LTO", "off"),
            (env_keys::BUILD_RS_ABI_STEP_HINT, "true"),
        ];
        [&compile_env, env].concat()
    };
    let dylib_artifact = cargo_native::compile::run::<Dylib>(
        &crate_metadata.manifest_path,
        cargo_args.as_slice(),
        compile_env,
        hide_warnings,
        color,
    )?;

    let mut contract_abi = pretty_print::handle_step("Extracting ABI...", || {
        let abi_entries = dylib::extract_abi_entries(&dylib_artifact)?;
        Ok(near_abi::__private::ChunkedAbiEntry::combine(abi_entries)?
            .into_abi_root(extract_metadata(crate_metadata)))
    })?;

    if !generate_docs {
        strip_docs(&mut contract_abi);
    }

    Ok(contract_abi)
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
