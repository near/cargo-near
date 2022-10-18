use crate::cargo::manifest::CargoManifestPath;
use crate::cargo::metadata::CrateMetadata;
use anyhow::Context;
use quote::{format_ident, quote};
use std::path::{Path, PathBuf};
use std::{collections::HashSet, fs};
use toml::value;

pub(crate) fn generate_toml(
    manifest_path: &CargoManifestPath,
    crate_metadata: &CrateMetadata,
) -> anyhow::Result<String> {
    let original_cargo_toml = fs::read_to_string(&manifest_path.path)?;
    let original_cargo_toml: toml::value::Table = toml::from_str(&original_cargo_toml)?;

    let mut near_sdk = original_cargo_toml
        .get("dependencies")
        .context("Cargo.toml [dependencies] section not found")?
        .get("near-sdk")
        .context("`near-sdk` dependency not found")?
        .as_table()
        .context("`near-sdk` dependency should be a table")?
        .clone();

    let cargo_toml = include_str!("../../templates/_Cargo.toml");
    let mut cargo_toml: toml::value::Table = toml::from_str(cargo_toml)?;
    let package = cargo_toml
        .get_mut("package")
        .context("Cargo.toml template [package] section not found")?
        .as_table_mut()
        .context("expected Cargo.toml template [package] section to be a table")?;
    package.insert(
        "name".to_string(),
        toml::value::Value::String(format!("{}-near-abi-gen", crate_metadata.root_package.name)),
    );
    let deps = cargo_toml
        .get_mut("dependencies")
        .context("Cargo.toml template [dependencies] section not found")?
        .as_table_mut()
        .context("expected Cargo.toml template [dependencies] section to be a table")?;

    // Make near-sdk dependency not use default features to save on compilation time, but ensure `abi` is enabled
    near_sdk.remove("optional");
    near_sdk.insert("default-features".to_string(), value::Value::Boolean(false));
    near_sdk.insert(
        "features".to_string(),
        value::Value::Array(vec![value::Value::String("abi".to_string())]),
    );

    // If near-sdk is a local path dependency, then convert the path to be absolute
    if let Some(near_sdk_path) = near_sdk.get_mut("path") {
        let path = near_sdk_path
            .as_str()
            .context("`near-sdk` path should be a string")?;
        let path = manifest_path.directory()?.join(PathBuf::from(path));
        *near_sdk_path = value::Value::String(path.canonicalize()?.to_string_lossy().into());
    }

    deps.insert("near-sdk".into(), near_sdk.into());

    let cargo_toml = toml::to_string(&cargo_toml)?;

    log::debug!("Cargo.toml contents:\n{}", &cargo_toml);

    Ok(cargo_toml)
}

pub(crate) fn generate_build_rs(dylib_path: &Path) -> anyhow::Result<String> {
    let dylib_dir = dylib_path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "Unable to infer the directory containing dylib file: {}",
            dylib_path.display()
        )
    })?;
    let dylib_name = dylib_path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Generated dylib is not a file: {}", dylib_path.display()))?
        .to_str()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unable to infer the directory containing dylib file: {}",
                dylib_path.display()
            )
        })?;

    let dylib_name = if let Some(dylib_name_stripped) = dylib_name.strip_prefix("lib") {
        dylib_name_stripped
    } else {
        anyhow::bail!(
            "Expected the generated dylib file to start with 'lib', but got '{}'",
            dylib_name
        );
    };

    let cargo_link_lib = format!("cargo:rustc-link-lib=dylib={}", &dylib_name);
    let cargo_link_search = format!("cargo:rustc-link-search=all={}", dylib_dir.display());

    let build_rs = quote! {
        fn main() {
            println!(#cargo_link_lib);
            println!(#cargo_link_search);
        }
    }
    .to_string();
    let build_rs_file = syn::parse_file(&build_rs).unwrap();
    let build_rs = prettyplease::unparse(&build_rs_file);

    log::debug!("build.rs contents:\n{}", &build_rs);

    Ok(build_rs)
}

pub(crate) fn generate_main_rs(dylib_path: &Path) -> anyhow::Result<String> {
    let dylib_file_contents = fs::read(&dylib_path)?;
    let object = symbolic_debuginfo::Object::parse(&dylib_file_contents)?;
    log::debug!(
        "A dylib was built at {:?} with format {} for architecture {}",
        &dylib_path,
        &object.file_format(),
        &object.arch()
    );
    let near_abi_symbols = object
        .symbols()
        .flat_map(|sym| sym.name)
        .filter(|sym_name| sym_name.starts_with("__near_abi_"))
        .map(|sym_name| sym_name.to_string())
        .collect::<HashSet<_>>();
    if near_abi_symbols.is_empty() {
        anyhow::bail!("No NEAR ABI symbols found in the dylib");
    }
    log::debug!("Detected NEAR ABI symbols: {:?}", &near_abi_symbols);

    let near_abi_function_defs = near_abi_symbols.iter().map(|s| {
        let name = format_ident!("{}", s);
        quote! {
            fn #name() -> near_sdk::__private::ChunkedAbiEntry;
        }
    });
    let near_abi_function_invocations = near_abi_symbols.iter().map(|s| {
        let name = format_ident!("{}", s);
        quote! {
            unsafe { #name() }
        }
    });

    let main_rs = quote! {
        extern "Rust" {
            #(#near_abi_function_defs)*
        }

        fn main() -> Result<(), std::io::Error> {
            let abi_entries: Vec<near_sdk::__private::ChunkedAbiEntry> = vec![#(#near_abi_function_invocations),*];
            let contents = serde_json::to_string_pretty(&abi_entries)?;
            print!("{}", contents);
            Ok(())
        }
    }
    .to_string();
    let main_rs_file = syn::parse_file(&main_rs).unwrap();
    let main_rs = prettyplease::unparse(&main_rs_file);

    log::debug!("main.rs contents:\n{}", &main_rs);

    Ok(main_rs)
}
