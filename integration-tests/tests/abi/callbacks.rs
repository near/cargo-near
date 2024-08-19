use cargo_near_build::near_abi::{AbiJsonParameter, AbiType};
use cargo_near_integration_tests::generate_abi_fn;
use function_name::named;
use schemars::gen::SchemaGenerator;

use crate::util::AsJsonSchema;

#[test]
#[named]
fn test_callbacks_unwrapped() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, #[callback_unwrap] a: bool, #[callback_unwrap] b: u32) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 0);
    assert_eq!(function.callbacks.len(), 2);
    let bool_schema = SchemaGenerator::default().subschema_for::<bool>();
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.callbacks[0],
        AbiType::Json {
            type_schema: bool_schema,
        }
    );
    assert_eq!(
        function.callbacks[1],
        AbiType::Json {
            type_schema: u32_schema,
        }
    );

    Ok(())
}

#[test]
#[named]
fn test_callbacks_result() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[callback_result] a: Result<String, near_sdk::PromiseError>,
            #[callback_result] b: Result<u32, near_sdk::PromiseError>
        ) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 0);
    assert_eq!(function.callbacks.len(), 2);
    let string_schema = SchemaGenerator::default().subschema_for::<String>();
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.callbacks[0],
        AbiType::Json {
            type_schema: string_schema,
        }
    );
    assert_eq!(
        function.callbacks[1],
        AbiType::Json {
            type_schema: u32_schema,
        }
    );

    Ok(())
}

#[test]
#[named]
fn test_callbacks_vec() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[callback_unwrap] a: bool,
            #[callback_vec] b: Vec<u32>
        ) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 0);
    assert_eq!(function.callbacks.len(), 1);
    let bool_schema = SchemaGenerator::default().subschema_for::<bool>();
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.callbacks[0],
        AbiType::Json {
            type_schema: bool_schema,
        }
    );
    assert_eq!(
        function.callbacks_vec,
        Some(AbiType::Json {
            type_schema: u32_schema,
        })
    );

    Ok(())
}

#[test]
#[named]
fn test_callbacks_mixed_with_params() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[callback_unwrap] a: bool,
            b: u32,
            #[callback_result] c: Result<String, near_sdk::PromiseError>,
            d: i32,
            #[callback_vec] e: Vec<u8>
        ) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);
    assert_eq!(function.callbacks.len(), 2);
    let bool_schema = SchemaGenerator::default().subschema_for::<bool>();
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    let string_schema = SchemaGenerator::default().subschema_for::<String>();
    let i32_schema = SchemaGenerator::default().subschema_for::<i32>();
    let u8_schema = SchemaGenerator::default().subschema_for::<u8>();
    assert_eq!(
        params[0],
        AbiJsonParameter {
            name: "b".to_string(),
            type_schema: u32_schema,
        }
    );
    assert_eq!(
        params[1],
        AbiJsonParameter {
            name: "d".to_string(),
            type_schema: i32_schema,
        }
    );
    assert_eq!(
        function.callbacks[0],
        AbiType::Json {
            type_schema: bool_schema,
        }
    );
    assert_eq!(
        function.callbacks[1],
        AbiType::Json {
            type_schema: string_schema,
        }
    );
    assert_eq!(
        function.callbacks_vec,
        Some(AbiType::Json {
            type_schema: u8_schema,
        })
    );

    Ok(())
}
