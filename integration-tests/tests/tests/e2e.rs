use cargo_near_integration_tests::generate_abi_fn;
use function_name::named;
use near_sdk::__private::{AbiFunction, AbiParameter, AbiType};
use schemars::gen::SchemaGenerator;
use std::fs;

#[test]
#[named]
fn test_simple_function() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function,
        &AbiFunction {
            name: "add".to_string(),
            doc: None,
            is_view: true,
            is_init: false,
            is_payable: false,
            is_private: false,
            params: vec![
                AbiParameter {
                    name: "a".to_string(),
                    typ: AbiType::Json {
                        type_schema: u32_schema.clone(),
                    }
                },
                AbiParameter {
                    name: "b".to_string(),
                    typ: AbiType::Json {
                        type_schema: u32_schema.clone(),
                    }
                }
            ],
            callbacks: vec![],
            callbacks_vec: None,
            result: Some(AbiType::Json {
                type_schema: u32_schema,
            })
        }
    );

    Ok(())
}
