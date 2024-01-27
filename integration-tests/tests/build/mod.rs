use crate::util;
use cargo_near_integration_tests::build_fn;
use function_name::named;

mod embed;
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
    let functions = build_result.abi_root.unwrap().body.functions;
    assert_eq!(functions.len(), 2);
    assert_eq!(functions[0].name, "add");
    assert_eq!(functions[0].doc, None);

    assert!(build_result.abi_compressed.is_none());

    util::test_add(&build_result.wasm).await?;

    Ok(())
}
