use crate::cargo::manifest::CargoManifestPath;
use quote::{format_ident, quote};
use std::path::{Path, PathBuf};
use std::{collections::HashSet, fs};
use toml::value;

pub(crate) fn generate_toml(manifest_path: &CargoManifestPath) -> anyhow::Result<String> {
    let original_cargo_toml = fs::read_to_string(&manifest_path.path)?;
    let original_cargo_toml: toml::value::Table = toml::from_str(&original_cargo_toml)?;

    let mut near_sdk = original_cargo_toml
        .get("dependencies")
        .ok_or_else(|| anyhow::anyhow!("[dependencies] section not found"))?
        .get("near-sdk")
        .ok_or_else(|| anyhow::anyhow!("near-sdk dependency not found"))?
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("near-sdk dependency should be a table"))?
        .clone();

    if !near_sdk
        .get("features")
        .and_then(|features| features.as_array())
        .map(|features| features.contains(&value::Value::String("abi".to_string())))
        .unwrap_or(false)
    {
        anyhow::bail!("Unable to generate ABI: NEAR SDK \"abi\" feature is not enabled")
    }

    let cargo_toml = include_str!("../../templates/_Cargo.toml");
    let mut cargo_toml: toml::value::Table = toml::from_str(cargo_toml)?;
    let deps = cargo_toml
        .get_mut("dependencies")
        .expect("[dependencies] section specified in the template")
        .as_table_mut()
        .expect("[dependencies] is a table specified in the template");

    // Make near-sdk dependency use default features
    near_sdk.remove("default-features");
    near_sdk.remove("optional");
    near_sdk.insert(
        "features".to_string(),
        value::Value::Array(vec![value::Value::String("abi".to_string())]),
    );

    // Convert NEAR SDK path to absolute
    if let Some(near_sdk_path) = near_sdk.get_mut("path") {
        let path = near_sdk_path
            .as_str()
            .expect("NEAR SDK path should be a string");
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
    log::debug!("Detected NEAR ABI symbols: {:?}", &near_abi_symbols);

    let near_abi_function_defs = near_abi_symbols.iter().map(|s| {
        let name = format_ident!("{}", s);
        quote! {
            fn #name() -> near_sdk::__private::AbiRoot;
        }
    });
    let near_abi_function_invocations = near_abi_symbols.iter().map(|s| {
        let name = format_ident!("{}", s);
        quote! {
            #name()
        }
    });

    let main_rs = quote! {
        extern "Rust" {
            #(#near_abi_function_defs)*
        }

        fn main() -> Result<(), std::io::Error> {
            let root_abis = unsafe { vec![#(#near_abi_function_invocations),*] };
            let combined_root_abi = near_sdk::__private::AbiRoot::combine(root_abis);
            let contents = serde_json::to_string_pretty(&combined_root_abi)?;
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
