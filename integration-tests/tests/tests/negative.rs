use cargo_near_integration_tests::generate_abi_fn_with;
use function_name::named;

#[test]
#[named]
fn test_abi_feature_not_enabled() -> anyhow::Result<()> {
    fn run_test() -> anyhow::Result<()> {
        generate_abi_fn_with! {
            Cargo: "/templates/_Cargo_no_abi_feature.toml";
            Code:
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
