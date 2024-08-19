use cargo_near_build::near_abi::{
    AbiFunction, AbiFunctionKind, AbiJsonParameter, AbiParameters, AbiType,
};
use cargo_near_integration_tests::generate_abi_fn;
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
