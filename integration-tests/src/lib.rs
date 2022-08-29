#[macro_export]
macro_rules! generate_abi {
    (with Cargo $cargo_path:expr, and vars $cargo_vars:expr; $($code:tt)*) => {{
        let manifest_dir: std::path::PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let workspace_dir = manifest_dir.parent().unwrap().join("target").join("_abi-integration-tests");
        let crate_dir = workspace_dir.join(function_name!());
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let mut cargo_toml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $cargo_path)).to_string();
        let mut vars: std::collections::HashMap<&str, &str> = $cargo_vars;
        vars.insert("name", function_name!());
        for (k, v) in vars {
            cargo_toml = cargo_toml.replace(&format!("::{}::", k), v);
        }
        let cargo_path = crate_dir.join("Cargo.toml");
        fs::write(&cargo_path, cargo_toml)?;

        let lib_rs_file = syn::parse_file(&quote::quote! { $($code)* }.to_string()).unwrap();
        let lib_rs = prettyplease::unparse(&lib_rs_file);
        let lib_rs_path = src_dir.join("lib.rs");
        fs::write(lib_rs_path, lib_rs)?;

        cargo_near::exec(cargo_near::NearCommand::Abi(cargo_near::AbiCommand {
            manifest_path: Some(cargo_path),
            doc: false,
            out_dir: None,
        }))?;

        let abi_root: near_abi::AbiRoot =
        serde_json::from_slice(&fs::read(workspace_dir.join("target").join("near").join(format!("{}_abi.json", function_name!())))?)?;
        abi_root
    }};
    (with Cargo $cargo_path:expr; $($code:tt)*) => {
        $crate::generate_abi! { with Cargo $cargo_path, and vars std::collections::HashMap::new(); $($code)* }
    };
    ($($code:tt)*) => {
        $crate::generate_abi! { with Cargo "/templates/_Cargo.toml"; $($code)* }
    }
}

/// Generate ABI for one function
#[macro_export]
macro_rules! generate_abi_fn {
    (with Cargo $cargo_path:expr, and vars $cargo_vars:expr; $($code:tt)*) => {{
        $crate::generate_abi! {
            with Cargo $cargo_path, and vars $cargo_vars;
            use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize, BorshSchema};
            use near_sdk::near_bindgen;

            #[near_bindgen]
            #[derive(Default, BorshDeserialize, BorshSerialize)]
            pub struct Contract {}

            #[near_bindgen]
            impl Contract {
                $($code)*
            }
        }
    }};
    (with Cargo $cargo_path:expr; $($code:tt)*) => {
        $crate::generate_abi_fn! { with Cargo $cargo_path, and vars std::collections::HashMap::new(); $($code)* }
    };
    ($($code:tt)*) => {
        $crate::generate_abi_fn! { with Cargo "/templates/_Cargo.toml"; $($code)* }
    }
}
