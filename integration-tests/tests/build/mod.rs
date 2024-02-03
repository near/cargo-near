use crate::util;
use cargo_near_integration_tests::build_fn;
use function_name::named;

mod opts;

#[tokio::test]
#[named]
async fn test_build_simple() -> cargo_near::CliResult {
    let build_result = build_fn! {
        /// Adds `a` and `b`.
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };
    let mut abi_root = build_result.abi_root.unwrap();
    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[0];
    assert_eq!(function.name, "add");
    assert_eq!(function.doc.as_ref().unwrap(), " Adds `a` and `b`.");

    // WASM hash is not included in the embedded ABI
    abi_root.metadata.wasm_hash = None;
    assert_eq!(
        util::fetch_contract_abi(&build_result.wasm).await?,
        abi_root
    );

    assert!(build_result.abi_compressed.is_some());

    util::test_add(&build_result.wasm).await?;

    Ok(())
}
