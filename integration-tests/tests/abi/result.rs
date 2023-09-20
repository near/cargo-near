use cargo_near_integration_tests::generate_abi_fn;
use function_name::named;
use near_abi::AbiType;
use schemars::gen::SchemaGenerator;

#[test]
#[named]
fn test_result_default() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert!(function.result.is_none());

    Ok(())
}

#[test]
#[named]
fn test_result_type() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self) -> u32 {
            1
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.result,
        Some(AbiType::Json {
            type_schema: u32_schema,
        })
    );

    Ok(())
}

#[test]
#[named]
fn test_result_handle_result() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        #[handle_result]
        pub fn foo(&self) -> Result<u32, &'static str> {
            Ok(1)
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let u32_schema = SchemaGenerator::default().subschema_for::<u32>();
    assert_eq!(
        function.result,
        Some(AbiType::Json {
            type_schema: u32_schema,
        })
    );

    Ok(())
}
