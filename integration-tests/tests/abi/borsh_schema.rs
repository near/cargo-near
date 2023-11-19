use crate::util::AsBorshSchema;
use borsh::schema::{BorshSchemaContainer, Definition, Fields};
use cargo_near_integration_tests::{generate_abi, generate_abi_fn};
use function_name::named;
use std::collections::BTreeMap;

#[test]
#[named]
fn test_borsh_schema_numeric_primitives_signed() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[serializer(borsh)] a: i8,
            #[serializer(borsh)] b: i16,
            #[serializer(borsh)] c: i32,
            #[serializer(borsh)] d: i64,
            #[serializer(borsh)] e: i128,
            #[serializer(borsh)] f: isize
        ) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 6);
    let i8_schema = BorshSchemaContainer::new(
        "i8".to_string(),
        BTreeMap::from([("i8".to_string(), Definition::Primitive(1))]),
    );
    let i16_schema = BorshSchemaContainer::new(
        "i16".to_string(),
        BTreeMap::from([("i16".to_string(), Definition::Primitive(2))]),
    );
    let i32_schema = BorshSchemaContainer::new(
        "i32".to_string(),
        BTreeMap::from([("i32".to_string(), Definition::Primitive(4))]),
    );
    let i64_schema = BorshSchemaContainer::new(
        "i64".to_string(),
        BTreeMap::from([("i64".to_string(), Definition::Primitive(8))]),
    );
    let i128_schema = BorshSchemaContainer::new(
        "i128".to_string(),
        BTreeMap::from([("i128".to_string(), Definition::Primitive(16))]),
    );
    let isize_schema = &i64_schema;
    assert_eq!(&params[0].type_schema, &i8_schema);
    assert_eq!(&params[1].type_schema, &i16_schema);
    assert_eq!(&params[2].type_schema, &i32_schema);
    assert_eq!(&params[3].type_schema, &i64_schema);
    assert_eq!(&params[4].type_schema, &i128_schema);
    assert_eq!(&params[5].type_schema, isize_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_numeric_primitives_unsigned() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[serializer(borsh)] a: u8,
            #[serializer(borsh)] b: u16,
            #[serializer(borsh)] c: u32,
            #[serializer(borsh)] d: u64,
            #[serializer(borsh)] e: u128,
            #[serializer(borsh)] f: usize
        ) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 6);
    let u8_schema = BorshSchemaContainer::new(
        "u8".to_string(),
        BTreeMap::from([("u8".to_string(), Definition::Primitive(1))]),
    );
    let u16_schema = BorshSchemaContainer::new(
        "u16".to_string(),
        BTreeMap::from([("u16".to_string(), Definition::Primitive(2))]),
    );
    let u32_schema = BorshSchemaContainer::new(
        "u32".to_string(),
        BTreeMap::from([("u32".to_string(), Definition::Primitive(4))]),
    );
    let u64_schema = BorshSchemaContainer::new(
        "u64".to_string(),
        BTreeMap::from([("u64".to_string(), Definition::Primitive(8))]),
    );
    let u128_schema = BorshSchemaContainer::new(
        "u128".to_string(),
        BTreeMap::from([("u128".to_string(), Definition::Primitive(16))]),
    );
    let usize_schema = &u64_schema;
    assert_eq!(&params[0].type_schema, &u8_schema);
    assert_eq!(&params[1].type_schema, &u16_schema);
    assert_eq!(&params[2].type_schema, &u32_schema);
    assert_eq!(&params[3].type_schema, &u64_schema);
    assert_eq!(&params[4].type_schema, &u128_schema);
    assert_eq!(&params[5].type_schema, usize_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_numeric_primitives_float() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, #[serializer(borsh)] a: f32, #[serializer(borsh)] b: f64) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 2);
    let f32_schema = BorshSchemaContainer::new(
        "f32".to_string(),
        BTreeMap::from([("f32".to_string(), Definition::Primitive(4))]),
    );
    let f64_schema = BorshSchemaContainer::new(
        "f64".to_string(),
        BTreeMap::from([("f64".to_string(), Definition::Primitive(8))]),
    );
    assert_eq!(&params[0].type_schema, &f32_schema);
    assert_eq!(&params[1].type_schema, &f64_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_string() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, #[serializer(borsh)] a: String, #[serializer(borsh)] b: &str, #[serializer(borsh)] c: &'static str) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 3);
    let string_schema = BorshSchemaContainer::new(
        "String".to_string(),
        BTreeMap::from([
            (
                "String".to_string(),
                Definition::Sequence {
                    length_width: Definition::DEFAULT_LENGTH_WIDTH,
                    length_range: Definition::DEFAULT_LENGTH_RANGE,
                    elements: "u8".to_string(),
                },
            ),
            ("u8".to_string(), Definition::Primitive(1)),
        ]),
    );
    assert_eq!(&params[0].type_schema, &string_schema);
    assert_eq!(&params[1].type_schema, &string_schema);
    assert_eq!(&params[2].type_schema, &string_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_other_primitives() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, #[serializer(borsh)] b: bool, #[serializer(borsh)] c: ()) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 2);
    // char is unsupported by borsh spec
    // let char_schema = BorshSchemaContainer {
    //     declaration: "char".to_string(),
    //     definitions: BTreeMap::new(),
    // };
    let bool_schema = BorshSchemaContainer::new(
        "bool".to_string(),
        BTreeMap::from([("bool".to_string(), Definition::Primitive(1))]),
    );
    let unit_schema = BorshSchemaContainer::new(
        "()".to_string(),
        BTreeMap::from([("()".to_string(), Definition::Primitive(0))]),
    );
    assert_eq!(&params[0].type_schema, &bool_schema);
    assert_eq!(&params[1].type_schema, &unit_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_tuples() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[serializer(borsh)] a: (bool,),
            #[serializer(borsh)] b: (bool, bool),
            #[serializer(borsh)] c: (bool, bool, bool)
        ) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 3);
    let tuple1_schema = BorshSchemaContainer::new(
        "(bool,)".to_string(),
        BTreeMap::from([
            (
                "(bool,)".to_string(),
                Definition::Tuple {
                    elements: vec!["bool".to_string()],
                },
            ),
            ("bool".to_string(), Definition::Primitive(1)),
        ]),
    );
    let tuple2_schema = BorshSchemaContainer::new(
        "(bool, bool)".to_string(),
        BTreeMap::from([
            (
                "(bool, bool)".to_string(),
                Definition::Tuple {
                    elements: vec!["bool".to_string(), "bool".to_string()],
                },
            ),
            ("bool".to_string(), Definition::Primitive(1)),
        ]),
    );
    let tuple3_schema = BorshSchemaContainer::new(
        "(bool, bool, bool)".to_string(),
        BTreeMap::from([
            (
                "(bool, bool, bool)".to_string(),
                Definition::Tuple {
                    elements: vec!["bool".to_string(), "bool".to_string(), "bool".to_string()],
                },
            ),
            ("bool".to_string(), Definition::Primitive(1)),
        ]),
    );
    assert_eq!(&params[0].type_schema, &tuple1_schema);
    assert_eq!(&params[1].type_schema, &tuple2_schema);
    assert_eq!(&params[2].type_schema, &tuple3_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_arrays() -> cargo_near::CliResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(
            &self,
            #[serializer(borsh)] a: [bool; 8],
            #[serializer(borsh)] b: [bool; 16],
            #[serializer(borsh)] c: &[bool]
        ) {}
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 3);
    let array8_schema = BorshSchemaContainer::new(
        "[bool; 8]".to_string(),
        BTreeMap::from([
            (
                "[bool; 8]".to_string(),
                Definition::Sequence {
                    length_width: Definition::ARRAY_LENGTH_WIDTH,
                    length_range: 8..=8,
                    elements: "bool".to_string(),
                },
            ),
            ("bool".to_string(), Definition::Primitive(1)),
        ]),
    );
    let array16_schema = BorshSchemaContainer::new(
        "[bool; 16]".to_string(),
        BTreeMap::from([
            (
                "[bool; 16]".to_string(),
                Definition::Sequence {
                    length_width: Definition::ARRAY_LENGTH_WIDTH,
                    length_range: 16..=16,
                    elements: "bool".to_string(),
                },
            ),
            ("bool".to_string(), Definition::Primitive(1)),
        ]),
    );
    let array_unlim_schema = BorshSchemaContainer::new(
        "Vec<bool>".to_string(),
        BTreeMap::from([
            (
                "Vec<bool>".to_string(),
                Definition::Sequence {
                    length_width: Definition::DEFAULT_LENGTH_WIDTH,
                    length_range: Definition::DEFAULT_LENGTH_RANGE,
                    elements: "bool".to_string(),
                },
            ),
            ("bool".to_string(), Definition::Primitive(1)),
        ]),
    );
    assert_eq!(&params[0].type_schema, &array8_schema);
    assert_eq!(&params[1].type_schema, &array16_schema);
    assert_eq!(&params[2].type_schema, &array_unlim_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_struct() -> cargo_near::CliResult {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
        use near_sdk::{near_bindgen, NearSchema};

        #[derive(NearSchema, BorshSerialize, BorshDeserialize)]
        #[abi(borsh)]
        #[borsh(crate = "near_sdk::borsh")]
        pub struct Pair(u32, u32);

        #[derive(NearSchema, BorshSerialize, BorshDeserialize)]
        #[abi(borsh)]
        #[borsh(crate = "near_sdk::borsh")]
        pub struct PairNamed {
            first: u32,
            second: u32
        }

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        #[borsh(crate = "near_sdk::borsh")]
        pub struct Contract {}

        #[near_bindgen]
        impl Contract {
            pub fn foo(&self, #[serializer(borsh)] a: Pair, #[serializer(borsh)] b: PairNamed) {}
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 2);
    let pair_def_schema = BorshSchemaContainer::new(
        "Pair".to_string(),
        BTreeMap::from([
            (
                "Pair".to_string(),
                Definition::Struct {
                    fields: Fields::UnnamedFields(vec!["u32".to_string(), "u32".to_string()]),
                },
            ),
            ("u32".to_string(), Definition::Primitive(4)),
        ]),
    );
    let pair_named_def_schema = BorshSchemaContainer::new(
        "PairNamed".to_string(),
        BTreeMap::from([
            (
                "PairNamed".to_string(),
                Definition::Struct {
                    fields: Fields::NamedFields(vec![
                        ("first".to_string(), "u32".to_string()),
                        ("second".to_string(), "u32".to_string()),
                    ]),
                },
            ),
            ("u32".to_string(), Definition::Primitive(4)),
        ]),
    );
    assert_eq!(&params[0].type_schema, &pair_def_schema);
    assert_eq!(&params[1].type_schema, &pair_named_def_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_enum() -> cargo_near::CliResult {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
        use near_sdk::{near_bindgen, NearSchema};

        #[derive(NearSchema, BorshSerialize, BorshDeserialize)]
        #[abi(borsh)]
        #[borsh(crate = "near_sdk::borsh")]
        pub enum IpAddrKind {
            V4,
            V6,
        }

        #[derive(NearSchema, BorshSerialize, BorshDeserialize)]
        #[abi(borsh)]
        #[borsh(crate = "near_sdk::borsh")]
        pub enum IpAddr {
            V4(u8, u8, u8, u8),
            V6(String),
        }

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        #[borsh(crate = "near_sdk::borsh")]
        pub struct Contract {}

        #[near_bindgen]
        impl Contract {
            pub fn foo(&self, #[serializer(borsh)] a: IpAddrKind, #[serializer(borsh)] b: IpAddr) {}
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 2);
    let ip_addr_kind_def_schema = BorshSchemaContainer::new(
        "IpAddrKind".to_string(),
        BTreeMap::from([
            (
                "IpAddrKind".to_string(),
                Definition::Enum {
                    tag_width: 1,
                    variants: vec![
                        (0, "V4".to_string(), "IpAddrKindV4".to_string()),
                        (1, "V6".to_string(), "IpAddrKindV6".to_string()),
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
    );
    let ip_addr_def_schema = BorshSchemaContainer::new(
        "IpAddr".to_string(),
        BTreeMap::from([
            (
                "IpAddr".to_string(),
                Definition::Enum {
                    tag_width: 1,
                    variants: vec![
                        (0, "V4".to_string(), "IpAddrV4".to_string()),
                        (1, "V6".to_string(), "IpAddrV6".to_string()),
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
                    fields: Fields::UnnamedFields(vec!["String".to_string()]),
                },
            ),
            (
                "String".to_string(),
                Definition::Sequence {
                    length_width: Definition::DEFAULT_LENGTH_WIDTH,
                    length_range: Definition::DEFAULT_LENGTH_RANGE,
                    elements: "u8".to_string(),
                },
            ),
            ("u8".to_string(), Definition::Primitive(1)),
        ]),
    );
    assert_eq!(&params[0].type_schema, &ip_addr_kind_def_schema);
    assert_eq!(&params[1].type_schema, &ip_addr_def_schema);

    Ok(())
}

#[test]
#[named]
fn test_borsh_schema_complex() -> cargo_near::CliResult {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
        use near_sdk::{near_bindgen, NearSchema};

        #[derive(NearSchema, BorshSerialize, BorshDeserialize)]
        #[abi(borsh)]
        #[borsh(crate = "near_sdk::borsh")]
        pub enum IpAddrKind {
            V4,
            V6,
        }

        #[derive(NearSchema, BorshSerialize, BorshDeserialize)]
        #[abi(borsh)]
        #[borsh(crate = "near_sdk::borsh")]
        pub struct IpAddr {
            kind: IpAddrKind,
            address: String,
        }

        #[near_bindgen]
        #[derive(Default, BorshDeserialize, BorshSerialize)]
        #[borsh(crate = "near_sdk::borsh")]
        pub struct Contract {}

        #[near_bindgen]
        impl Contract {
            pub fn foo(&self, #[serializer(borsh)] b: IpAddr) {}
        }
    };

    assert_eq!(abi_root.body.functions.len(), 1);
    let function = &abi_root.body.functions[0];
    let params = function.params.borsh_schemas()?;
    assert_eq!(params.len(), 1);
    let ip_addr_def_schema = BorshSchemaContainer::new(
        "IpAddr".to_string(),
        BTreeMap::from([
            (
                "IpAddr".to_string(),
                Definition::Struct {
                    fields: Fields::NamedFields(vec![
                        ("kind".to_string(), "IpAddrKind".to_string()),
                        ("address".to_string(), "String".to_string()),
                    ]),
                },
            ),
            (
                "IpAddrKind".to_string(),
                Definition::Enum {
                    tag_width: 1,
                    variants: vec![
                        (0, "V4".to_string(), "IpAddrKindV4".to_string()),
                        (1, "V6".to_string(), "IpAddrKindV6".to_string()),
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
            (
                "String".to_string(),
                Definition::Sequence {
                    length_width: Definition::DEFAULT_LENGTH_WIDTH,
                    length_range: Definition::DEFAULT_LENGTH_RANGE,
                    elements: "u8".to_string(),
                },
            ),
            ("u8".to_string(), Definition::Primitive(1)),
        ]),
    );
    assert_eq!(&params[0].type_schema, &ip_addr_def_schema);

    Ok(())
}
