use cargo_near_integration_tests::generate_abi_fn;
use function_name::named;
use std::fs;

#[test]
#[named]
fn test_abi_feature_not_enabled() -> anyhow::Result<()> {
    fn run_test() -> anyhow::Result<()> {
        generate_abi_fn! {
            with Cargo "/templates/_Cargo_no_abi_feature.toml";
            pub fn foo(&self, #[callback_unwrap] a: bool, #[callback_unwrap] b: u32) {}
        };
        Ok(())
    }

    assert_eq!(
        run_test().unwrap_err().to_string(),
        "Unable to generate ABI: NEAR SDK \"abi\" feature is not enabled"
    );

    Ok(())
}
