use cargo_near_integration_tests::generate_abi_fn;
use function_name::named;
use std::fs;

#[test]
#[named]
fn test_view_function() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert!(function.is_view);
    assert!(!function.is_init);
    assert!(!function.is_payable);
    assert!(!function.is_private);

    Ok(())
}

#[test]
#[named]
fn test_call_function() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn add(&mut self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert!(!function.is_view);

    Ok(())
}

#[test]
#[named]
fn test_init_function() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        #[init]
        pub fn add(a: u32, b: u32) -> Self {
            Contract {}
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert!(function.is_init);
    assert!(function.result.is_none());

    Ok(())
}

#[test]
#[named]
fn test_payable_function() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        #[payable]
        pub fn add(&mut self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert!(function.is_payable);

    Ok(())
}

#[test]
#[named]
fn test_private_function() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        #[private]
        pub fn add(&self, a: u32, b: u32) -> u32 {
            a + b
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    assert!(function.is_private);

    Ok(())
}
