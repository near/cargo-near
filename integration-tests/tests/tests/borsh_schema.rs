use borsh::schema::{BorshSchemaContainer, Definition, Fields};
use cargo_near_integration_tests::{generate_abi, generate_abi_fn};
use function_name::named;
use near_abi::{AbiParameter, AbiType};
use std::{collections::HashMap, fs};

trait AsBorshSchema {
    fn borsh_schema(&self) -> anyhow::Result<&BorshSchemaContainer>;
}

impl AsBorshSchema for AbiParameter {
    fn borsh_schema(&self) -> anyhow::Result<&BorshSchemaContainer> {
        if let AbiType::Borsh { type_schema } = &self.typ {
            Ok(type_schema)
        } else {
            anyhow::bail!("Expected Borsh serialization type, but got {:?}", self)
        }
    }
}

#[test]
#[named]
fn test_borsh_schema_numeric_primitives_signed() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[serializer(borsh)] a: i8,
            #[serializer(borsh)] b: i16,
            #[serializer(borsh)] c: i32,
            #[serializer(borsh)] d: i64,
            #[serializer(borsh)] e: i128
        ) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 5);
    let i8_schema = BorshSchemaContainer {
        declaration: "i8".to_string(),
        definitions: HashMap::new(),
    };
    let i16_schema = BorshSchemaContainer {
        declaration: "i16".to_string(),
        definitions: HashMap::new(),
    };
    let i32_schema = BorshSchemaContainer {
        declaration: "i32".to_string(),
        definitions: HashMap::new(),
    };
    let i64_schema = BorshSchemaContainer {
        declaration: "i64".to_string(),
        definitions: HashMap::new(),
    };
    let i128_schema = BorshSchemaContainer {
        declaration: "i128".to_string(),
        definitions: HashMap::new(),
    };
    // borsh-rs does not support isize schema; need https://github.com/near/borsh-rs/pull/99 to come in first
    // let isize_schema = BorshSchemaContainer {
    //     declaration: "i64".to_string(),
    //     definitions: HashMap::new(),
    // };
    assert_eq!(function.params[0].borsh_schema()?, &i8_schema);
    assert_eq!(function.params[1].borsh_schema()?, &i16_schema);
    assert_eq!(function.params[2].borsh_schema()?, &i32_schema);
    assert_eq!(function.params[3].borsh_schema()?, &i64_schema);
    assert_eq!(function.params[4].borsh_schema()?, &i128_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_numeric_primitives_unsigned() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[serializer(borsh)] a: u8,
            #[serializer(borsh)] b: u16,
            #[serializer(borsh)] c: u32,
            #[serializer(borsh)] d: u64,
            #[serializer(borsh)] e: u128
        ) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 5);
    let u8_schema = BorshSchemaContainer {
        declaration: "u8".to_string(),
        definitions: HashMap::new(),
    };
    let u16_schema = BorshSchemaContainer {
        declaration: "u16".to_string(),
        definitions: HashMap::new(),
    };
    let u32_schema = BorshSchemaContainer {
        declaration: "u32".to_string(),
        definitions: HashMap::new(),
    };
    let u64_schema = BorshSchemaContainer {
        declaration: "u64".to_string(),
        definitions: HashMap::new(),
    };
    let u128_schema = BorshSchemaContainer {
        declaration: "u128".to_string(),
        definitions: HashMap::new(),
    };
    // borsh-rs does not support usize schema; need https://github.com/near/borsh-rs/pull/99 to come in first
    // let usize_schema = BorshSchemaContainer {
    //     declaration: "u64".to_string(),
    //     definitions: HashMap::new(),
    // };
    assert_eq!(function.params[0].borsh_schema()?, &u8_schema);
    assert_eq!(function.params[1].borsh_schema()?, &u16_schema);
    assert_eq!(function.params[2].borsh_schema()?, &u32_schema);
    assert_eq!(function.params[3].borsh_schema()?, &u64_schema);
    assert_eq!(function.params[4].borsh_schema()?, &u128_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_numeric_primitives_float() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, #[serializer(borsh)] a: f32, #[serializer(borsh)] b: f64) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 2);
    let f32_schema = BorshSchemaContainer {
        declaration: "f32".to_string(),
        definitions: HashMap::new(),
    };
    let f64_schema = BorshSchemaContainer {
        declaration: "f64".to_string(),
        definitions: HashMap::new(),
    };
    assert_eq!(function.params[0].borsh_schema()?, &f32_schema);
    assert_eq!(function.params[1].borsh_schema()?, &f64_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_string() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, #[serializer(borsh)] a: String, #[serializer(borsh)] b: &str, #[serializer(borsh)] c: &'static str) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 3);
    let string_schema = BorshSchemaContainer {
        declaration: "string".to_string(),
        definitions: HashMap::new(),
    };
    assert_eq!(function.params[0].borsh_schema()?, &string_schema);
    assert_eq!(function.params[1].borsh_schema()?, &string_schema);
    assert_eq!(function.params[2].borsh_schema()?, &string_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_other_primitives() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, #[serializer(borsh)] b: bool, #[serializer(borsh)] c: ()) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 2);
    // char is unsupported by borsh spec
    // let char_schema = BorshSchemaContainer {
    //     declaration: "char".to_string(),
    //     definitions: HashMap::new(),
    // };
    let bool_schema = BorshSchemaContainer {
        declaration: "bool".to_string(),
        definitions: HashMap::new(),
    };
    let unit_schema = BorshSchemaContainer {
        declaration: "nil".to_string(),
        definitions: HashMap::new(),
    };
    assert_eq!(function.params[0].borsh_schema()?, &bool_schema);
    assert_eq!(function.params[1].borsh_schema()?, &unit_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_tuples() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[serializer(borsh)] b: (bool, bool),
            #[serializer(borsh)] c: (bool, bool, bool)
        ) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 2);
    // Needs https://github.com/near/borsh-rs/pull/100 to come in first
    // let tuple1_schema = BorshSchemaContainer {
    //     declaration: "Tuple<bool>".to_string(),
    //     definitions: HashMap::new(),
    // };
    let tuple2_schema = BorshSchemaContainer {
        declaration: "Tuple<bool, bool>".to_string(),
        definitions: HashMap::from([(
            "Tuple<bool, bool>".to_string(),
            Definition::Tuple {
                elements: vec!["bool".to_string(), "bool".to_string()],
            },
        )]),
    };
    let tuple3_schema = BorshSchemaContainer {
        declaration: "Tuple<bool, bool, bool>".to_string(),
        definitions: HashMap::from([(
            "Tuple<bool, bool, bool>".to_string(),
            Definition::Tuple {
                elements: vec!["bool".to_string(), "bool".to_string(), "bool".to_string()],
            },
        )]),
    };
    assert_eq!(function.params[0].borsh_schema()?, &tuple2_schema);
    assert_eq!(function.params[1].borsh_schema()?, &tuple3_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_arrays() -> anyhow::Result<()> {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[serializer(borsh)] a: [bool; 8],
            #[serializer(borsh)] b: [bool; 16],
            #[serializer(borsh)] c: &[bool]
        ) {}
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 3);
    let array8_schema = BorshSchemaContainer {
        declaration: "Array<bool, 8>".to_string(),
        definitions: HashMap::from([(
            "Array<bool, 8>".to_string(),
            Definition::Array {
                length: 8,
                elements: "bool".to_string(),
            },
        )]),
    };
    let array16_schema = BorshSchemaContainer {
        declaration: "Array<bool, 16>".to_string(),
        definitions: HashMap::from([(
            "Array<bool, 16>".to_string(),
            Definition::Array {
                length: 16,
                elements: "bool".to_string(),
            },
        )]),
    };
    let array_unlim_schema = BorshSchemaContainer {
        declaration: "Vec<bool>".to_string(),
        definitions: HashMap::from([(
            "Vec<bool>".to_string(),
            Definition::Sequence {
                elements: "bool".to_string(),
            },
        )]),
    };
    assert_eq!(function.params[0].borsh_schema()?, &array8_schema);
    assert_eq!(function.params[1].borsh_schema()?, &array16_schema);
    assert_eq!(function.params[2].borsh_schema()?, &array_unlim_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_struct() -> anyhow::Result<()> {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize, BorshSchema};
        use near_sdk::near_bindgen;

        #[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
        pub struct Pair(u32, u32);

        #[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
        pub struct PairNamed {
            first: u32,
            second: u32
        }

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        pub struct Contract {}

        #[near_bindgen]
        impl Contract {
            pub fn foo(&self, #[serializer(borsh)] a: Pair, #[serializer(borsh)] b: PairNamed) {}
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 2);
    let pair_def_schema = BorshSchemaContainer {
        declaration: "Pair".to_string(),
        definitions: HashMap::from([(
            "Pair".to_string(),
            Definition::Struct {
                fields: Fields::UnnamedFields(vec!["u32".to_string(), "u32".to_string()]),
            },
        )]),
    };
    let pair_named_def_schema = BorshSchemaContainer {
        declaration: "PairNamed".to_string(),
        definitions: HashMap::from([(
            "PairNamed".to_string(),
            Definition::Struct {
                fields: Fields::NamedFields(vec![
                    ("first".to_string(), "u32".to_string()),
                    ("second".to_string(), "u32".to_string()),
                ]),
            },
        )]),
    };
    assert_eq!(function.params[0].borsh_schema()?, &pair_def_schema);
    assert_eq!(function.params[1].borsh_schema()?, &pair_named_def_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_enum() -> anyhow::Result<()> {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize, BorshSchema};
        use near_sdk::near_bindgen;

        #[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
        pub enum IpAddrKind {
            V4,
            V6,
        }

        #[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
        pub enum IpAddr {
            V4(u8, u8, u8, u8),
            V6(String),
        }

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        pub struct Contract {}

        #[near_bindgen]
        impl Contract {
            pub fn foo(&self, #[serializer(borsh)] a: IpAddrKind, #[serializer(borsh)] b: IpAddr) {}
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 2);
    let ip_addr_kind_def_schema = BorshSchemaContainer {
        declaration: "IpAddrKind".to_string(),
        definitions: HashMap::from([
            (
                "IpAddrKind".to_string(),
                Definition::Enum {
                    variants: vec![
                        ("V4".to_string(), "IpAddrKindV4".to_string()),
                        ("V6".to_string(), "IpAddrKindV6".to_string()),
                    ],
                },
            ),
            (
                "IpAddrKindV4".to_string(),
                Definition::Struct {
                    fields: Fields::Empty,
                },
            ),
            (
                "IpAddrKindV6".to_string(),
                Definition::Struct {
                    fields: Fields::Empty,
                },
            ),
        ]),
    };
    let ip_addr_def_schema = BorshSchemaContainer {
        declaration: "IpAddr".to_string(),
        definitions: HashMap::from([
            (
                "IpAddr".to_string(),
                Definition::Enum {
                    variants: vec![
                        ("V4".to_string(), "IpAddrV4".to_string()),
                        ("V6".to_string(), "IpAddrV6".to_string()),
                    ],
                },
            ),
            (
                "IpAddrV4".to_string(),
                Definition::Struct {
                    fields: Fields::UnnamedFields(vec![
                        "u8".to_string(),
                        "u8".to_string(),
                        "u8".to_string(),
                        "u8".to_string(),
                    ]),
                },
            ),
            (
                "IpAddrV6".to_string(),
                Definition::Struct {
                    fields: Fields::UnnamedFields(vec!["string".to_string()]),
                },
            ),
        ]),
    };
    assert_eq!(function.params[0].borsh_schema()?, &ip_addr_kind_def_schema);
    assert_eq!(function.params[1].borsh_schema()?, &ip_addr_def_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_complex() -> anyhow::Result<()> {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize, BorshSchema};
        use near_sdk::near_bindgen;

        #[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
        pub enum IpAddrKind {
            V4,
            V6,
        }

        #[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
        pub struct IpAddr {
            kind: IpAddrKind,
            address: String,
        }

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        pub struct Contract {}

        #[near_bindgen]
        impl Contract {
            pub fn foo(&self, #[serializer(borsh)] b: IpAddr) {}
        }
    };

    assert_eq!(abi_root.abi.functions.len(), 1);
    let function = &abi_root.abi.functions[0];
    assert_eq!(function.params.len(), 1);
    let ip_addr_def_schema = BorshSchemaContainer {
        declaration: "IpAddr".to_string(),
        definitions: HashMap::from([
            (
                "IpAddr".to_string(),
                Definition::Struct {
                    fields: Fields::NamedFields(vec![
                        ("kind".to_string(), "IpAddrKind".to_string()),
                        ("address".to_string(), "string".to_string()),
                    ]),
                },
            ),
            (
                "IpAddrKind".to_string(),
                Definition::Enum {
                    variants: vec![
                        ("V4".to_string(), "IpAddrKindV4".to_string()),
                        ("V6".to_string(), "IpAddrKindV6".to_string()),
                    ],
                },
            ),
            (
                "IpAddrKindV4".to_string(),
                Definition::Struct {
                    fields: Fields::Empty,
                },
            ),
            (
                "IpAddrKindV6".to_string(),
                Definition::Struct {
                    fields: Fields::Empty,
                },
            ),
        ]),
    };
    assert_eq!(function.params[0].borsh_schema()?, &ip_addr_def_schema);

    Ok(())
}
