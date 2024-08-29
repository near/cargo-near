/// NOTE: `near-sdk` version, published on crates.io
pub mod from_crates_io {
    use const_format::formatcp;

    pub const SDK_VERSION: &str = "5.2.1";
    pub const SDK_VERSION_TOML: &str = formatcp!(r#"version = "{SDK_VERSION}""#);
}

/// NOTE: this version is version of near-sdk in arbitrary revision from N.x.x development cycle
pub mod from_git {
    use const_format::formatcp;

    pub const SDK_VERSION: &str = "5.2.1";
    pub const SDK_REVISION: &str = "5c9eaeaad596f42df7002446beca63f819791c5f";
    pub const SDK_SHORT_VERSION_TOML: &str = formatcp!(r#"version = "{SDK_VERSION}""#);
    pub const SDK_REPO: &str = "https://github.com/near/near-sdk-rs.git";
    pub const SDK_VERSION_TOML: &str =
        formatcp!(r#"version = "{SDK_VERSION}", git = "{SDK_REPO}", rev = "{SDK_REVISION}""#);
    pub const SDK_VERSION_TOML_TABLE: &str = formatcp!(
        r#"
        version = "{SDK_VERSION}"
        git = "https://github.com/near/near-sdk-rs.git"
        rev = "{SDK_REVISION}"
        "#
    );
}

#[macro_export]
macro_rules! invoke_cargo_near {
    ($(Cargo: $cargo_path:expr;)? $(Vars: $cargo_vars:expr;)? Opts: $cli_opts:expr; Code: $($code:tt)*) => {{
        let manifest_dir: camino::Utf8PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let workspace_dir = manifest_dir.parent().unwrap().join("target").join("_abi-integration-tests");
        let crate_dir = workspace_dir.join(function_name!());
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;

        let mut cargo_toml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/_Cargo.toml")).to_string();
        $(cargo_toml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $cargo_path)).to_string())?;
        let mut cargo_vars = std::collections::HashMap::new();
        $(cargo_vars = $cargo_vars)?;
        cargo_vars.insert("sdk-cratesio-version", $crate::from_crates_io::SDK_VERSION);
        cargo_vars.insert("sdk-cratesio-version-toml", $crate::from_crates_io::SDK_VERSION_TOML);
        cargo_vars.insert("sdk-git-version", $crate::from_git::SDK_VERSION);
        cargo_vars.insert("sdk-git-short-version-toml", $crate::from_git::SDK_SHORT_VERSION_TOML);
        cargo_vars.insert("sdk-git-version-toml", $crate::from_git::SDK_VERSION_TOML);
        cargo_vars.insert("sdk-git-version-toml-table", $crate::from_git::SDK_VERSION_TOML_TABLE);
        cargo_vars.insert("name", function_name!());
        for (k, v) in cargo_vars {
            cargo_toml = cargo_toml.replace(&format!("::{}::", k), v);
        }
        let cargo_path = crate_dir.join("Cargo.toml");
        std::fs::write(&cargo_path, cargo_toml)?;

        let lib_rs_file = syn::parse_file(&quote::quote! { $($code)* }.to_string()).unwrap();
        let lib_rs = prettyplease::unparse(&lib_rs_file);
        let lib_rs_path = src_dir.join("lib.rs");
        std::fs::write(lib_rs_path, lib_rs)?;

        std::env::set_var("CARGO_TARGET_DIR", workspace_dir.join("target"));

        let cargo_near::CliOpts::Near(cli_args) = cargo_near::Opts::try_parse_from($cli_opts.split(" "))?;

        match cli_args.cmd {
            Some(cargo_near::commands::CliNearCommand::Abi(cmd)) => {
                let args = cargo_near_build::abi::AbiOpts {
                    no_locked: cmd.no_locked,
                    no_doc: cmd.no_doc,
                    compact_abi: cmd.compact_abi,
                    out_dir: cmd.out_dir.map(Into::into),
                    manifest_path: Some(cargo_path.into()),
                    color: cmd.color.map(Into::into),
                };
                cargo_near_build::abi::build(args)?;
            },
            Some(cargo_near::commands::CliNearCommand::Build(cmd)) => {
                let args = {
                  let mut args = cargo_near::commands::build_command::BuildCommand::from(cmd)  ;
                  args.manifest_path = Some(cargo_path.into());
                  args
                };
                args.run(cargo_near_build::BuildContext::Build)?;
            },
            Some(_) => todo!(),
            None => ()
        }

        workspace_dir.join("target").join("near")
    }};
}

#[macro_export]
macro_rules! generate_abi_with {
    ($(Cargo: $cargo_path:expr;)? $(Vars: $cargo_vars:expr;)? $(Opts: $cli_opts:expr;)? Code: $($code:tt)*) => {{
        let opts = "cargo near abi --no-locked";
        $(let opts = format!("cargo near abi --no-locked {}", $cli_opts);)?;
        let result_dir = $crate::invoke_cargo_near! {
            $(Cargo: $cargo_path;)? $(Vars: $cargo_vars;)?
            Opts: &opts;
            Code:
            $($code)*
        };

        let abi_root: cargo_near_build::near_abi::AbiRoot =
            serde_json::from_slice(&std::fs::read(result_dir.join(format!("{}_abi.json", function_name!())))?)?;
        abi_root
    }};
}

#[macro_export]
macro_rules! generate_abi {
    ($($code:tt)*) => {{
        $crate::generate_abi_with! {
            Code:
            $($code)*
        }
    }};
}

/// Generate ABI for one function
#[macro_export]
macro_rules! generate_abi_fn_with {
    ($(Cargo: $cargo_path:expr;)? $(Vars: $cargo_vars:expr;)? $(Opts: $cli_opts:expr;)? Code: $($code:tt)*) => {{
        $crate::generate_abi_with! {
            $(Cargo: $cargo_path;)? $(Vars: $cargo_vars;)? $(Opts: $cli_opts;)?
            Code:
            use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
            use near_sdk::near_bindgen;

            #[near_bindgen]
            #[derive(Default, BorshDeserialize, BorshSerialize)]
            #[borsh(crate = "near_sdk::borsh")]
            pub struct Contract {}

            #[near_bindgen]
            impl Contract {
                $($code)*
            }
        }
    }};
}

/// Generate ABI for one function
#[macro_export]
macro_rules! generate_abi_fn {
    ($($code:tt)*) => {{
        $crate::generate_abi_fn_with! {
            Code:
            $($code)*
        }
    }};
}

pub struct BuildResult {
    pub wasm: Vec<u8>,
    pub abi_root: Option<cargo_near_build::near_abi::AbiRoot>,
    pub abi_compressed: Option<Vec<u8>>,
}

// TODO: make cargo-near agnostic of stdin/stdout and capture the resulting paths from Writer
#[macro_export]
macro_rules! build_with {
    ($(Cargo: $cargo_path:expr;)? $(Vars: $cargo_vars:expr;)? $(Opts: $cli_opts:expr;)? Code: $($code:tt)*) => {{
        let opts = "cargo near build --no-docker --no-locked";
        $(let opts = format!("cargo near build --no-docker --no-locked {}", $cli_opts);)?;
        let result_dir = $crate::invoke_cargo_near! {
            $(Cargo: $cargo_path;)? $(Vars: $cargo_vars;)?
            Opts: &opts;
            Code:
            $($code)*
        };

        let manifest_dir: std::path::PathBuf = env!("CARGO_MANIFEST_DIR").into();
        let workspace_dir = manifest_dir.parent().unwrap().join("target").join("_abi-integration-tests");
        let wasm_debug_path = workspace_dir.join("target")
            .join("wasm32-unknown-unknown")
            .join("debug")
            .join(format!("{}.wasm", function_name!()));
        let wasm_release_path = workspace_dir.join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(format!("{}.wasm", function_name!()));
        let wasm: Vec<u8> = if wasm_release_path.exists() {
            std::fs::read(wasm_release_path)?
        } else {
            std::fs::read(wasm_debug_path)?
        };

        let abi_path = result_dir.join(format!("{}_abi.json", function_name!()));
        let abi_root: Option<cargo_near_build::near_abi::AbiRoot> = if abi_path.exists() {
            Some(serde_json::from_slice(&std::fs::read(abi_path)?)?)
        } else {
            None
        };

        let abi_compressed_path = result_dir.join(format!("{}_abi.zst", function_name!()));
        let abi_compressed: Option<Vec<u8>> = if abi_compressed_path.exists() {
            Some(std::fs::read(abi_compressed_path)?)
        } else {
            None
        };

        $crate::BuildResult { wasm, abi_root, abi_compressed }
    }};
}

#[macro_export]
macro_rules! build {
    ($($code:tt)*) => {{
        $crate::build_with! {
            Code:
            $($code)*
        }
    }};
}

#[macro_export]
macro_rules! build_fn_with {
    ($(Cargo: $cargo_path:expr;)? $(Vars: $cargo_vars:expr;)? $(Opts: $cli_opts:expr;)? Code: $($code:tt)*) => {{
        $crate::build_with! {
            $(Cargo: $cargo_path;)? $(Vars: $cargo_vars;)? $(Opts: $cli_opts;)?
            Code:
            use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
            use near_sdk::{near_bindgen, NearSchema};

            #[near_bindgen]
            #[derive(Default, BorshDeserialize, BorshSerialize)]
            #[borsh(crate = "near_sdk::borsh")]
            pub struct Contract {}

            #[near_bindgen]
            impl Contract {
                $($code)*
            }
        }
    }};
}

#[macro_export]
macro_rules! build_fn {
    ($($code:tt)*) => {{
        $crate::build_fn_with! {
            Code:
            $($code)*
        }
    }};
}
