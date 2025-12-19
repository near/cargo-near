use crate::util::AsJsonSchema;
use cargo_near_integration_tests::{generate_abi, generate_abi_fn};
use function_name::named;
use schemars::schema::Schema;

#[test]
#[named]
fn test_schema_numeric_primitives_signed() -> testresult::TestResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, a: i8, b: i16, c: i32, d: i64, e: i128, f: isize) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 6);
    // `format` is an open-ended keyword so one can define their own custom formats.
    // See https://json-schema.org/draft/2020-12/json-schema-validation.html#name-custom-format-attributes.
    // We make use of it to annotate `integer` type with specific numeric formats. This has some
    // interoperability issues though as, for instance, JavaScript would not be able to map JSON
    // numbers to `i64` and `i128` formats.
    let i8_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "int8"
        }
        "#,
    )?;
    let i16_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "int16"
        }
        "#,
    )?;
    let i32_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "int32"
        }
        "#,
    )?;
    let i64_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "int64"
        }
        "#,
    )?;
    let i128_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "int128"
        }
        "#,
    )?;
    let isize_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "int"
        }
        "#,
    )?;
    assert_eq!(&params[0].type_schema, &i8_schema);
    assert_eq!(&params[1].type_schema, &i16_schema);
    assert_eq!(&params[2].type_schema, &i32_schema);
    assert_eq!(&params[3].type_schema, &i64_schema);
    assert_eq!(&params[4].type_schema, &i128_schema);
    assert_eq!(&params[5].type_schema, &isize_schema);

    Ok(())
}

#[test]
#[named]
fn test_schema_numeric_primitives_unsigned() -> testresult::TestResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, a: u8, b: u16, c: u32, d: u64, e: u128, f: usize) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 6);
    // `format` is an open-ended keyword so one can define their own custom formats.
    // See https://json-schema.org/draft/2020-12/json-schema-validation.html#name-custom-format-attributes.
    // We make use of it to annotate `integer` type with specific numeric formats. This has some
    // interoperability issues though as, for instance, JavaScript would not be able to map JSON
    // numbers to `u64` and `u128` formats.
    let u8_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "uint8",
            "minimum": 0.0
        }
        "#,
    )?;
    let u16_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "uint16",
            "minimum": 0.0
        }
        "#,
    )?;
    let u32_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "uint32",
            "minimum": 0.0
        }
        "#,
    )?;
    let u64_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "uint64",
            "minimum": 0.0
        }
        "#,
    )?;
    let u128_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "uint128",
            "minimum": 0.0
        }
        "#,
    )?;
    let usize_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "integer",
            "format": "uint",
            "minimum": 0.0
        }
        "#,
    )?;
    assert_eq!(&params[0].type_schema, &u8_schema);
    assert_eq!(&params[1].type_schema, &u16_schema);
    assert_eq!(&params[2].type_schema, &u32_schema);
    assert_eq!(&params[3].type_schema, &u64_schema);
    assert_eq!(&params[4].type_schema, &u128_schema);
    assert_eq!(&params[5].type_schema, &usize_schema);

    Ok(())
}

#[test]
#[named]
fn test_schema_numeric_primitives_float() -> testresult::TestResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, a: f32, b: f64) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);
    // `format` is an open-ended keyword so one can define their own custom formats.
    // See https://json-schema.org/draft/2020-12/json-schema-validation.html#name-custom-format-attributes.
    // We make use of it to annotate `integer` type with specific numeric formats. This has some
    // interoperability issues though as, for instance, JavaScript would not be able to map JSON
    // numbers to `f64` format.
    let f32_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "number",
            "format": "float"
        }
        "#,
    )?;
    let f64_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "number",
            "format": "double"
        }
        "#,
    )?;
    assert_eq!(&params[0].type_schema, &f32_schema);
    assert_eq!(&params[1].type_schema, &f64_schema);

    Ok(())
}

#[test]
#[named]
fn test_schema_string() -> testresult::TestResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, a: String, b: &str, c: &'static str) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 3);
    let string_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "string"
        }
        "#,
    )?;
    assert_eq!(&params[0].type_schema, &string_schema);
    assert_eq!(&params[1].type_schema, &string_schema);
    assert_eq!(&params[2].type_schema, &string_schema);

    Ok(())
}

#[test]
#[named]
fn test_schema_other_primitives() -> testresult::TestResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, a: char, b: bool, c: ()) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 3);
    let char_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "string",
            "maxLength": 1,
            "minLength": 1
        }
        "#,
    )?;
    let bool_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "boolean"
        }
        "#,
    )?;
    let unit_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "null"
        }
        "#,
    )?;
    assert_eq!(&params[0].type_schema, &char_schema);
    assert_eq!(&params[1].type_schema, &bool_schema);
    assert_eq!(&params[2].type_schema, &unit_schema);

    Ok(())
}

#[test]
#[named]
fn test_schema_tuples() -> testresult::TestResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, a: (bool,), b: (bool, bool), c: (bool, bool, bool)) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 3);
    let tuple1_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "array",
            "items": [
              {
                "type": "boolean"
              }
            ],
            "maxItems": 1,
            "minItems": 1
        }
        "#,
    )?;
    let tuple2_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "array",
            "items": [
              {
                "type": "boolean"
              },
              {
                "type": "boolean"
              }
            ],
            "maxItems": 2,
            "minItems": 2
        }
        "#,
    )?;
    let tuple3_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "array",
            "items": [
              {
                "type": "boolean"
              },
              {
                "type": "boolean"
              },
              {
                "type": "boolean"
              }
            ],
            "maxItems": 3,
            "minItems": 3
        }
        "#,
    )?;
    assert_eq!(&params[0].type_schema, &tuple1_schema);
    assert_eq!(&params[1].type_schema, &tuple2_schema);
    assert_eq!(&params[2].type_schema, &tuple3_schema);

    Ok(())
}

#[test]
#[named]
fn test_schema_arrays() -> testresult::TestResult {
    let abi_root = generate_abi_fn! {
        pub fn foo(&self, a: [bool; 8], b: [bool; 16], c: &[bool]) {}
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 3);
    let array8_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "array",
            "items": {
                "type": "boolean"
            },
            "maxItems": 8,
            "minItems": 8
        }
        "#,
    )?;
    let array16_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "array",
            "items": {
                "type": "boolean"
            },
            "maxItems": 16,
            "minItems": 16
        }
        "#,
    )?;
    let array_unlim_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "array",
            "items": {
                "type": "boolean"
            }
        }
        "#,
    )?;
    assert_eq!(&params[0].type_schema, &array8_schema);
    assert_eq!(&params[1].type_schema, &array16_schema);
    assert_eq!(&params[2].type_schema, &array_unlim_schema);

    Ok(())
}

#[test]
#[named]
fn test_schema_struct() -> testresult::TestResult {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
        use near_sdk::{near_bindgen, NearSchema};
        use near_sdk::serde::{Deserialize, Serialize};

        #[derive(NearSchema, Serialize, Deserialize)]
        #[abi(json)]
        pub struct Pair(u32, u32);

        #[derive(NearSchema, Serialize, Deserialize)]
        #[abi(json)]
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
            pub fn foo(&self, a: Pair, b: PairNamed) {}
        }
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);
    let pair_def_schema: Schema = serde_json::from_str(
        r##"
        {
            "$ref": "#/definitions/Pair"
        }
        "##,
    )?;
    let pair_named_def_schema: Schema = serde_json::from_str(
        r##"
        {
            "$ref": "#/definitions/PairNamed"
        }
        "##,
    )?;
    assert_eq!(&params[0].type_schema, &pair_def_schema);
    assert_eq!(&params[1].type_schema, &pair_named_def_schema);

    // Structs with unnamed parameters are serialized as arrays, hence they are represented as
    // arrays in JSON Schema.
    let pair_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "array",
            "items": [
                {
                    "type": "integer",
                    "format": "uint32",
                    "minimum": 0.0
                },
                {
                    "type": "integer",
                    "format": "uint32",
                    "minimum": 0.0
                }
            ],
            "maxItems": 2,
            "minItems": 2
        }
        "#,
    )?;
    let pair_named_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "object",
            "required": [
                "first",
                "second"
            ],
            "properties": {
                "first": {
                    "type": "integer",
                    "format": "uint32",
                    "minimum": 0.0
                },
                "second": {
                    "type": "integer",
                    "format": "uint32",
                    "minimum": 0.0
                }
            }
        }
        "#,
    )?;
    assert_eq!(abi_root.body.root_schema.definitions["Pair"], pair_schema);
    assert_eq!(
        abi_root.body.root_schema.definitions["PairNamed"],
        pair_named_schema
    );

    Ok(())
}

#[test]
#[named]
fn test_schema_enum() -> testresult::TestResult {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
        use near_sdk::{near_bindgen, NearSchema};
        use near_sdk::serde::{Deserialize, Serialize};

        #[derive(NearSchema, Serialize, Deserialize)]
        #[abi(json)]
        pub enum IpAddrKind {
            V4,
            V6,
        }

        #[derive(NearSchema, Serialize, Deserialize)]
        #[abi(json)]
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
            pub fn foo(&self, a: IpAddrKind, b: IpAddr) {}
        }
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);
    let ip_addr_kind_def_schema: Schema = serde_json::from_str(
        r##"
        {
            "$ref": "#/definitions/IpAddrKind"
        }
        "##,
    )?;
    let ip_addr_def_schema: Schema = serde_json::from_str(
        r##"
        {
            "$ref": "#/definitions/IpAddr"
        }
        "##,
    )?;
    assert_eq!(&params[0].type_schema, &ip_addr_kind_def_schema);
    assert_eq!(&params[1].type_schema, &ip_addr_def_schema);

    let ip_addr_kind_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "string",
            "enum": [
                "V4",
                "V6"
            ]
        }
        "#,
    )?;
    let ip_addr_schema: Schema = serde_json::from_str(
        r#"
        {
            "oneOf": [
                {
                    "type": "object",
                    "required": [
                        "V4"
                    ],
                    "properties": {
                        "V4": {
                            "type": "array",
                            "items": [
                                {
                                    "type": "integer",
                                    "format": "uint8",
                                    "minimum": 0.0
                                },
                                {
                                    "type": "integer",
                                    "format": "uint8",
                                    "minimum": 0.0
                                },
                                {
                                    "type": "integer",
                                    "format": "uint8",
                                    "minimum": 0.0
                                },
                                {
                                    "type": "integer",
                                    "format": "uint8",
                                    "minimum": 0.0
                                }
                            ],
                            "maxItems": 4,
                            "minItems": 4
                        }
                    },
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "required": [
                        "V6"
                    ],
                    "properties": {
                        "V6": {
                            "type": "string"
                        }
                    },
                    "additionalProperties": false
                }
            ]
        }
        "#,
    )?;
    assert_eq!(
        abi_root.body.root_schema.definitions["IpAddrKind"],
        ip_addr_kind_schema
    );
    assert_eq!(
        abi_root.body.root_schema.definitions["IpAddr"],
        ip_addr_schema
    );

    Ok(())
}

#[test]
#[named]
fn test_schema_complex() -> testresult::TestResult {
    let abi_root = generate_abi! {
        use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
        use near_sdk::{near_bindgen, NearSchema};
        use near_sdk::serde::{Deserialize, Serialize};

        #[derive(NearSchema, Serialize, Deserialize)]
        #[abi(json)]
        pub enum IpAddrKind {
            V4,
            V6,
        }

        #[derive(NearSchema, Serialize, Deserialize)]
        #[abi(json)]
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
            pub fn foo(&self, a: IpAddrKind, b: IpAddr) {}
        }
    };

    assert_eq!(abi_root.body.functions.len(), 2);
    let function = &abi_root.body.functions[1];
    let params = function.params.json_schemas()?;
    assert_eq!(params.len(), 2);
    let ip_addr_kind_def_schema: Schema = serde_json::from_str(
        r##"
        {
            "$ref": "#/definitions/IpAddrKind"
        }
        "##,
    )?;
    let ip_addr_def_schema: Schema = serde_json::from_str(
        r##"
        {
            "$ref": "#/definitions/IpAddr"
        }
        "##,
    )?;
    assert_eq!(&params[0].type_schema, &ip_addr_kind_def_schema);
    assert_eq!(&params[1].type_schema, &ip_addr_def_schema);

    let ip_addr_kind_schema: Schema = serde_json::from_str(
        r#"
        {
            "type": "string",
            "enum": [
                "V4",
                "V6"
            ]
        }
        "#,
    )?;
    let ip_addr_schema: Schema = serde_json::from_str(
        r##"
        {
            "type": "object",
            "required": [
                "address",
                "kind"
            ],
            "properties": {
                "address": {
                    "type": "string"
                },
                "kind": {
                    "$ref": "#/definitions/IpAddrKind"
                }
            }
        }
        "##,
    )?;
    assert_eq!(
        abi_root.body.root_schema.definitions["IpAddrKind"],
        ip_addr_kind_schema
    );
    assert_eq!(
        abi_root.body.root_schema.definitions["IpAddr"],
        ip_addr_schema
    );

    Ok(())
}
