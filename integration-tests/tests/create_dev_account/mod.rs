use cargo_near_integration_tests::create_dev_account_fn;
use function_name::named;

#[test]
#[named]
fn test_generate_random_account_id() -> cargo_near::CliResult {
    create_dev_account_fn! {
        Cmd: "cargo near create-dev-account";
        Code:
    };
    Ok(())
}
