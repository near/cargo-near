use cargo_near::{AbiCommand, NearCommand};
use near_sdk::__private::{AbiFunction, AbiParameter, AbiRoot, AbiSerializationType, AbiType};
use proc_macro2::TokenStream;
use quote::quote;
use schemars::gen::SchemaGenerator;
use std::fs;

fn generate_abi(code: TokenStream) -> anyhow::Result<AbiRoot> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_dir_path = tmp_dir.path();
    let src_dir_path = tmp_dir_path.join("src");
    fs::create_dir_all(&src_dir_path)?;

    let cargo_toml = include_str!("templates/_Cargo.toml");
    let cargo_path = tmp_dir_path.join("Cargo.toml");
    fs::write(&cargo_path, cargo_toml)?;

    let lib_rs_file = syn::parse_file(&code.to_string()).unwrap();
    let lib_rs = prettyplease::unparse(&lib_rs_file);
    let lib_rs_path = src_dir_path.join("lib.rs");
    fs::write(lib_rs_path, lib_rs)?;

    cargo_near::exec(NearCommand::Abi(AbiCommand {
        manifest_path: Some(cargo_path),
    }))?;

    let abi_root = serde_json::from_slice(&fs::read(tmp_dir_path.join("target/near/abi.json"))?)?;
    Ok(abi_root)
}

#[test]
fn test_simple_function() -> anyhow::Result<()> {
    let code = quote! {
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
    let abi_root = generate_abi(code)?;

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
