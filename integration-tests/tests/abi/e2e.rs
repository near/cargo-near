use cargo_near_build::near_abi::{
    AbiFunction, AbiFunctionKind, AbiJsonParameter, AbiParameters, AbiType,
};
use cargo_near_integration_tests::{generate_abi_fn, generate_abi_with};
use function_name::named;
use schemars::r#gen::SchemaGenerator;

#[test]
#[named]
fn test_simple_function() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[0];
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function,
        &AbiFunction {
            name: "add".to_string(),
            doc: None,
            kind: AbiFunctionKind::View,
            modifiers: vec![],
            params: AbiParameters::Json {
                args: vec![
                    AbiJsonParameter {
                        name: "a".to_string(),
                        type_schema: u32_schema.clone(),
                    },
                    AbiJsonParameter {
                        name: "b".to_string(),
                        type_schema: u32_schema.clone(),
                    }
                ],
            },
            callbacks: vec![],
            callbacks_vec: None,
            result: Some(AbiType::Json {
                type_schema: u32_schema,
            })
        }
    );

    Ok(())
}

#[test]
#[named]
fn test_abi_with_unguarded_no_mangle_function() -> cargo_near::CliResult {
    let abi_root = generate_abi_with! {
        Code:
        use near_sdk::near;

        #[near(contract_state)]
        #[derive(Default)]
        pub struct Contract {}

        #[near]
        impl Contract {
            pub fn get_value(&self) -> u32 {
                42
            }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn custom_function() {
            unsafe {
                near_sdk::sys::input(0);
            }
        }
    };

    let function_names = abi_root
        .body
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();

    assert!(function_names.contains(&"get_value"));
    assert!(!function_names.contains(&"custom_function"));

    Ok(())
}
