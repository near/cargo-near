use cargo_near_integration_tests::{generate_abi_fn, generate_abi_fn_with};
use function_name::named;

#[test]
#[named]
#[should_panic(
    expected = "missing `__abi-generate` required feature for `near-sdk` dependency: probably unsupported `near-sdk` version. expected 4.1.* or higher"
)]
fn test_abi_old_sdk() {
    generate_abi_fn_with! {
        Cargo: "/templates/negative/_Cargo_old_sdk.toml";
        Code:
        pub fn foo(&self, a: u32, b: u32) {}
    };
}

#[test]
#[named]
#[should_panic(
    expected = "Error invoking `cargo metadata`. Your `Cargo.toml` file is likely malformed"
)]
fn test_abi_weird_version() {
    generate_abi_fn_with! {
        Cargo: "/templates/negative/_Cargo_malformed.toml";
        Code:
        pub fn foo(&self, a: u32, b: u32) {}
    };
}
