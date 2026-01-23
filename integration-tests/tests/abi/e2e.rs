use cargo_near_build::near_abi::{
    AbiFunction, AbiFunctionKind, AbiJsonParameter, AbiParameters, AbiType,
};
use cargo_near_integration_tests::{generate_abi_fn, generate_abi};
use function_name::named;
use schemars::gen::SchemaGenerator;

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
fn test_no_mangle_with_cfg() -> cargo_near::CliResult {
    // This test verifies that ABI generation works with #[no_mangle] functions
    // that are conditionally compiled using #[cfg(not(near_abi))]
    let abi_root = generate_abi! {
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

        // This function should not cause linker errors during ABI generation
        // because it's excluded with #[cfg(not(near_abi))]
        #[cfg(not(near_abi))]
        #[no_mangle]
        pub extern "C" fn custom_function() {
            // This would normally cause linker errors during ABI generation
            // if not excluded, because it might call NEAR host functions
        }
    };

    // Verify ABI was generated successfully
    // Expected: 2 functions (get_value + __contract_abi metadata function)
    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.name, "get_value");

    Ok(())
}
