use cargo_near_integration_tests::generate_abi_fn;
use function_name::named;
use near_sdk::__private::{AbiParameter, AbiSerializationType, AbiType};
use schemars::gen::SchemaGenerator;
use std::fs;

#[test]
#[named]
fn test_callbacks_unwrapped() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, #[callback_unwrap] a: bool, #[callback_unwrap] b: u32) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 0);
    assert_eq!(function.callbacks.len(), 2);
    let bool_schema = SchemaGenerator::default().subschema_for::<bool>();
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.callbacks[0],
        AbiType {
            type_schema: bool_schema,
            serialization_type: AbiSerializationType::Json
        }
    );
    assert_eq!(
        function.callbacks[1],
        AbiType {
            type_schema: u32_schema,
            serialization_type: AbiSerializationType::Json
        }
    );

    Ok(())
}

#[test]
#[named]
fn test_callbacks_result() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[callback_result] a: Result<String, near_sdk::PromiseError>,
            #[callback_result] b: Result<u32, near_sdk::PromiseError>
        ) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 0);
    assert_eq!(function.callbacks.len(), 2);
    let string_schema = SchemaGenerator::default().subschema_for::<String>();
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.callbacks[0],
        AbiType {
            type_schema: string_schema,
            serialization_type: AbiSerializationType::Json
        }
    );
    assert_eq!(
        function.callbacks[1],
        AbiType {
            type_schema: u32_schema,
            serialization_type: AbiSerializationType::Json
        }
    );

    Ok(())
}

#[test]
#[named]
fn test_callbacks_vec() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[callback_unwrap] a: bool,
            #[callback_vec] b: Vec<u32>
        ) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 0);
    assert_eq!(function.callbacks.len(), 1);
    let bool_schema = SchemaGenerator::default().subschema_for::<bool>();
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.callbacks[0],
        AbiType {
            type_schema: bool_schema,
            serialization_type: AbiSerializationType::Json
        }
    );
    assert_eq!(
        function.callbacks_vec,
        Some(AbiType {
            type_schema: u32_schema,
            serialization_type: AbiSerializationType::Json
        })
    );

    Ok(())
}

#[test]
#[named]
fn test_callbacks_mixed_with_params() -> anyhow::Result<()> {
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

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 2);
    assert_eq!(function.callbacks.len(), 2);
    let bool_schema = SchemaGenerator::default().subschema_for::<bool>();
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    let string_schema = SchemaGenerator::default().subschema_for::<String>();
    let i32_schema = SchemaGenerator::default().subschema_for::<i32>();
    let u8_schema = SchemaGenerator::default().subschema_for::<u8>();
    assert_eq!(
        function.params[0],
        AbiParameter {
            name: "b".to_string(),
            type_schema: u32_schema,
            serialization_type: AbiSerializationType::Json,
        }
    );
    assert_eq!(
        function.params[1],
        AbiParameter {
            name: "d".to_string(),
            type_schema: i32_schema,
            serialization_type: AbiSerializationType::Json,
        }
    );
    assert_eq!(
        function.callbacks[0],
        AbiType {
            type_schema: bool_schema,
            serialization_type: AbiSerializationType::Json
        }
    );
    assert_eq!(
        function.callbacks[1],
        AbiType {
            type_schema: string_schema,
            serialization_type: AbiSerializationType::Json
        }
    );
    assert_eq!(
        function.callbacks_vec,
        Some(AbiType {
            type_schema: u8_schema,
            serialization_type: AbiSerializationType::Json
        })
    );

    Ok(())
}
