use cargo_near_integration_tests::{generate_abi_fn, generate_abi_fn_with};
use function_name::named;

#[test]
#[named]
fn test_abi_old_sdk() -> anyhow::Result<()> {
    fn run_test() -> anyhow::Result<()> {
        generate_abi_fn_with! {
            Cargo: "/templates/negative/_Cargo_old_sdk.toml";
            Code:
            pub fn foo(&self, a: u32, b: u32) {}
        };
        Ok(())
    }

    assert_eq!(
        run_test().unwrap_err().to_string(),
        "unsupported `near-sdk` version. expected 4.1.* or higher"
    );

    Ok(())
}

#[test]
#[named]
fn test_abi_weird_version() -> anyhow::Result<()> {
    fn run_test() -> anyhow::Result<()> {
        generate_abi_fn_with! {
            Cargo: "/templates/negative/_Cargo_malformed.toml";
            Code:
            pub fn foo(&self, a: u32, b: u32) {}
        };
        Ok(())
    }

    assert_eq!(
        run_test().unwrap_err().to_string(),
        "Error invoking `cargo metadata`. Your `Cargo.toml` file is likely malformed"
    );

    Ok(())
}

// TODO: Arguably this should not be an error. Feels like generating ABI for a contract
// with no code should work.
#[test]
#[named]
fn test_abi_no_code() -> anyhow::Result<()> {
    fn run_test() -> anyhow::Result<()> {
        generate_abi_fn! {};
        Ok(())
    }

    assert_eq!(
        run_test().unwrap_err().to_string(),
        "No NEAR ABI symbols found in the dylib"
    );

    Ok(())
}
