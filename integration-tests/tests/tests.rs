use cargo_near::{AbiCommand, NearCommand};
use function_name::named;
use near_sdk::__private::{AbiFunction, AbiParameter, AbiRoot, AbiSerializationType, AbiType};
use schemars::gen::SchemaGenerator;
use std::{fs, path::PathBuf};

macro_rules! generate_abi {
    ($($code:tt)*) => {{
        let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let workspace_dir = manifest_dir.parent().unwrap().join("target").join("_abi-integration-tests");
        let crate_dir = workspace_dir.join(function_name!());
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let cargo_toml = include_str!("templates/_Cargo_workspace.toml");
        fs::write(&workspace_dir.join("Cargo.toml"), cargo_toml)?;

        let cargo_toml = include_str!("templates/_Cargo.toml");
        let cargo_toml = cargo_toml.replace("::name::", function_name!());
        let cargo_path = crate_dir.join("Cargo.toml");
        fs::write(&cargo_path, cargo_toml)?;

        let lib_rs_file = syn::parse_file(&quote::quote! { $($code)* }.to_string()).unwrap();
        let lib_rs = prettyplease::unparse(&lib_rs_file);
        let lib_rs_path = src_dir.join("lib.rs");
        fs::write(lib_rs_path, lib_rs)?;

        cargo_near::exec(NearCommand::Abi(AbiCommand {
            manifest_path: Some(cargo_path),
        }))?;

        let abi_root: AbiRoot =
            serde_json::from_slice(&fs::read(workspace_dir.join("target").join("near").join(function_name!()).join("abi.json"))?)?;
        abi_root
    }};
}

#[test]
#[named]
fn test_view_function() -> anyhow::Result<()> {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
        use near_sdk::near_bindgen;

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        pub struct Contract {}

        #[near_bindgen]
        impl Contract {
            pub fn add(&self, a: u32, b: u32) -> u32 {
                a + b
            }
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function,
        &AbiFunction {
            name: "add".to_string(),
            is_view: true,
            is_init: false,
            is_payable: false,
            is_private: false,
            params: vec![
                AbiParameter {
                    name: "a".to_string(),
                    type_schema: u32_schema.clone(),
                    serialization_type: AbiSerializationType::Json
                },
                AbiParameter {
                    name: "b".to_string(),
                    type_schema: u32_schema.clone(),
                    serialization_type: AbiSerializationType::Json
                }
            ],
            callbacks: vec![],
            callbacks_vec: None,
            result: Some(AbiType {
                type_schema: u32_schema,
                serialization_type: AbiSerializationType::Json,
            })
        }
    );

    Ok(())
}

#[test]
#[named]
fn test_call_function() -> anyhow::Result<()> {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
        use near_sdk::near_bindgen;

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        pub struct Contract {
            state: u32
        }

        #[near_bindgen]
        impl Contract {
            pub fn add(&mut self, a: u32, b: u32) {
                self.state = a + b;
            }
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert!(!function.is_view);

    Ok(())
}

#[test]
#[named]
fn test_payable_function() -> anyhow::Result<()> {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
        use near_sdk::near_bindgen;

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        pub struct Contract {
            state: u32
        }

        #[near_bindgen]
        impl Contract {
            #[payable]
            pub fn add(&mut self, a: u32, b: u32) {
                self.state = a + b;
            }
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert!(function.is_payable);

    Ok(())
}

#[test]
#[named]
fn test_private_function() -> anyhow::Result<()> {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
        use near_sdk::near_bindgen;

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        pub struct Contract {
            state: u32
        }

        #[near_bindgen]
        impl Contract {
            #[private]
            pub fn add(&mut self, a: u32, b: u32) {
                self.state = a + b;
            }
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert!(function.is_private);

    Ok(())
}
