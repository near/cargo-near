use cargo_near_integration_tests::{generate_abi_fn, generate_abi_fn_with};
use function_name::named;

#[test]
#[named]
fn test_abi_feature_not_enabled() -> cargo_near::CliResult {
    fn run_test() -> cargo_near::CliResult {
        generate_abi_fn_with! {
            Cargo: "/templates/negative/_Cargo_no_abi_feature.toml";
            Code:
            pub fn foo(&self, a: u32, b: u32) {}
        };
        Ok(())
    }

    assert_eq!(
        run_test().map_err(|e| e.to_string()),
        Err("`near-sdk` dependency must have the `abi` feature enabled".to_string())
    );

    Ok(())
}

#[test]
#[named]
fn test_abi_old_sdk() -> cargo_near::CliResult {
    fn run_test() -> cargo_near::CliResult {
        generate_abi_fn_with! {
            Cargo: "/templates/negative/_Cargo_old_sdk.toml";
            Code:
            pub fn foo(&self, a: u32, b: u32) {}
        };
        Ok(())
    }

    assert_eq!(
        run_test().map_err(|e| e.to_string()),
        Err("unsupported `near-sdk` version. expected 4.1.* or higher".to_string())
    );

    Ok(())
}

#[test]
#[named]
fn test_abi_weird_version() -> cargo_near::CliResult {
    fn run_test() -> cargo_near::CliResult {
        generate_abi_fn_with! {
            Cargo: "/templates/negative/_Cargo_malformed.toml";
            Code:
            pub fn foo(&self, a: u32, b: u32) {}
        };
        Ok(())
    }

    assert_eq!(
        run_test().map_err(|e| e.to_string()),
        Err(
            "Error invoking `cargo metadata`. Your `Cargo.toml` file is likely malformed"
                .to_string()
        )
    );

    Ok(())
}

// TODO: Arguably this should not be an error. Feels like generating ABI for a contract
// with no code should work.
// NOTE: this was ignored, as abi now contains 
// ```json
// {
//   "name": "contract_source_metadata",
//   "kind": "view"
// }
// ```
// function by default
#[ignore]
#[test]
#[named]
fn test_abi_no_code() -> cargo_near::CliResult {
    fn run_test() -> cargo_near::CliResult {
        generate_abi_fn! {};
        Ok(())
    }

    assert_eq!(
        run_test().map_err(|e| e.to_string()),
        Err("No NEAR ABI symbols found in the dylib".to_string())
    );

    Ok(())
}
