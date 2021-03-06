#[macro_export]
macro_rules! generate_abi {
    ($($code:tt)*) => {{
        let manifest_dir: std::path::PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let workspace_dir = manifest_dir.parent().unwrap().join("target").join("_abi-integration-tests");
        let crate_dir = workspace_dir.join(function_name!());
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let cargo_toml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/_Cargo_workspace.toml"));
        fs::write(&workspace_dir.join("Cargo.toml"), cargo_toml)?;

        let cargo_toml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/_Cargo.toml"));
        let cargo_toml = cargo_toml.replace("::name::", function_name!());
        let cargo_path = crate_dir.join("Cargo.toml");
        fs::write(&cargo_path, cargo_toml)?;

        let lib_rs_file = syn::parse_file(&quote::quote! { $($code)* }.to_string()).unwrap();
        let lib_rs = prettyplease::unparse(&lib_rs_file);
        let lib_rs_path = src_dir.join("lib.rs");
        fs::write(lib_rs_path, lib_rs)?;

        cargo_near::exec(cargo_near::NearCommand::Abi(cargo_near::AbiCommand {
            manifest_path: Some(cargo_path),
        }))?;

        let abi_root: near_sdk::__private::AbiRoot =
            serde_json::from_slice(&fs::read(workspace_dir.join("target").join("near").join(function_name!()).join("abi.json"))?)?;
        abi_root
    }};
}

/// Generate ABI for one function
#[macro_export]
macro_rules! generate_abi_fn {
    ($($code:tt)*) => {
        $crate::generate_abi! {
            use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
            use near_sdk::near_bindgen;

            #[near_bindgen]
            #[derive(Default, BorshDeserialize, BorshSerialize)]
            pub struct Contract {}

            #[near_bindgen]
            impl Contract {
                $($code)*
            }
        }
    };
}
