use cargo_near_build::camino;

/// NOTE: `near-sdk` version, published on crates.io
pub mod from_crates_io {
    use const_format::formatcp;

    pub const SDK_VERSION: &str = "5.5.0";
    pub const SDK_VERSION_TOML: &str = formatcp!(r#"version = "{SDK_VERSION}""#);
}

pub fn setup_tracing() {
    use colored::Colorize;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::{fmt::format, prelude::*};
    let environment = "test".green();
    let my_formatter = cargo_near::types::my_formatter::MyFormatter::from_environment(environment);

    let format = format::debug_fn(move |writer, _field, value| write!(writer, "{:?}", value));

    let env_filter = EnvFilter::from_default_env();

    let _e = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(my_formatter)
                .fmt_fields(format)
                .with_filter(env_filter),
        )
        .try_init();
}

/// NOTE: this version is version of near-sdk in arbitrary revision from N.x.x development cycle
pub mod from_git {
    use const_format::formatcp;

    pub const SDK_VERSION: &str = "5.5.0";
    pub const SDK_REVISION: &str = "f5caed2af27e0c3984d34c3d1c943d6020b7ad2c";
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

pub fn common_root_for_test_projects_build() -> camino::Utf8PathBuf {
    let manifest_dir: camino::Utf8PathBuf = env!("CARGO_MANIFEST_DIR").into();
    let workspace_dir = manifest_dir
        .parent()
        .unwrap()
        .join("target")
        .join("_abi-integration-tests");
    workspace_dir
}

#[macro_export]
macro_rules! invoke_cargo_near {
    ($(Cargo: $cargo_path:expr;)? $(Vars: $cargo_vars:expr;)? Opts: $cli_opts:expr; Code: $($code:tt)*) => {{
        let workspace_dir = $crate::common_root_for_test_projects_build();
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

        let cargo_near::CliOpts::Near(cli_args) = cargo_near::Opts::try_parse_from($cli_opts.split(" "))?;

        let path: camino::Utf8PathBuf = match cli_args.cmd {
            Some(cargo_near::commands::CliNearCommand::Abi(cmd)) => {
                let args = cargo_near_build::abi::AbiOpts {
                    no_locked: cmd.no_locked,
                    no_doc: cmd.no_doc,
                    compact_abi: cmd.compact_abi,
                    out_dir: cmd.out_dir.map(Into::into),
                    manifest_path: Some(cargo_path.into()),
                    color: cmd.color.map(Into::into),
                };
                tracing::debug!("AbiOpts: {:#?}", args);
                let path = cargo_near_build::abi::build(args)?;
                path
            },
            Some(cargo_near::commands::CliNearCommand::Build(cmd)) => {
                let args = {
                  let mut args = cargo_near::commands::build_command::BuildCommand::from(cmd)  ;
                  args.manifest_path = Some(cargo_path.into());
                  args
                };
                tracing::debug!("BuildCommand: {:#?}", args);
                let artifact = args.run(cargo_near_build::BuildContext::Build)?;
                artifact.path
            },
            Some(_) => todo!(),
            None => unreachable!(),
        };
        path

    }};
}

#[macro_export]
macro_rules! generate_abi_with {
    ($(Cargo: $cargo_path:expr;)? $(Vars: $cargo_vars:expr;)? $(Opts: $cli_opts:expr;)? Code: $($code:tt)*) => {{
        let opts = "cargo near abi --no-locked";
        $(let opts = format!("cargo near abi --no-locked {}", $cli_opts);)?;
        let result_file = $crate::invoke_cargo_near! {
            $(Cargo: $cargo_path;)? $(Vars: $cargo_vars;)?
            Opts: &opts;
            Code:
            $($code)*
        };
        let result_dir = result_file.as_std_path().parent().expect("has parent");

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
        let result_file = $crate::invoke_cargo_near! {
            $(Cargo: $cargo_path;)? $(Vars: $cargo_vars;)?
            Opts: &opts;
            Code:
            $($code)*
        };
        let result_dir = result_file.as_std_path().parent().expect("has parent");

        let wasm_path = result_dir.
            join(format!("{}.wasm", function_name!()));
        let wasm: Vec<u8> = std::fs::read(wasm_path)?;

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
