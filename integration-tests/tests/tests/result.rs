use cargo_near_integration_tests::generate_abi_fn;
use function_name::named;
use near_sdk::__private::{AbiSerializationType, AbiType};
use schemars::gen::SchemaGenerator;
use std::fs;

#[test]
#[named]
fn test_result_default() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert!(function.result.is_none());

    Ok(())
}

#[test]
#[named]
fn test_result_type() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self) -> u32 {
            1
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.result,
        Some(AbiType {
            type_schema: u32_schema,
            serialization_type: AbiSerializationType::Json
        })
    );

    Ok(())
}

// FIXME: test does not work because handle_result type is not unwrapped from `Result<_, _>`.
// See https://github.com/near/near-sdk-rs/pull/858
#[test]
#[named]
#[ignore]
fn test_result_handle_result() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        #[handle_result]
        pub fn foo(&self) -> Result<u32, &'static str> {
            Ok(1)
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.result,
        Some(AbiType {
            type_schema: u32_schema,
            serialization_type: AbiSerializationType::Json
        })
    );

    Ok(())
}
