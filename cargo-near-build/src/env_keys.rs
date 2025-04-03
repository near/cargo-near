/// this is [`CARGO_TARGET_DIR`](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
pub const CARGO_TARGET_DIR: &str = "CARGO_TARGET_DIR";
/// this variable is set to `"true"` during ABI generation operation  
pub const BUILD_RS_ABI_STEP_HINT: &str = "CARGO_NEAR_ABI_GENERATION";

/// <https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags>
///
/// this behaviour that
/// 1. default value for RUSTFLAGS for wasm build is "-C link-arg=-s"
/// 2. it can be overridden with values from --env arguments
/// 3. default RUSTFLAGS for abi gen are "-Awarnings"
/// 4. RUSTFLAGS aren't concatenated (implicitly) with values from environment
///
/// is documented in RUSTFLAGS section of README.md
pub const RUSTFLAGS: &str = "RUSTFLAGS";

/// <https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags>
///
/// this behaviour that
/// 1. CARGO_ENCODED_RUSTFLAGS gets unset by default
///
/// is documented in CARGO_ENCODED_RUSTFLAGS section of README.md
pub const CARGO_ENCODED_RUSTFLAGS: &str = "CARGO_ENCODED_RUSTFLAGS";

pub(crate) const CARGO_NEAR_ABI_PATH: &str = "CARGO_NEAR_ABI_PATH";

pub(crate) const CARGO_NEAR_VERSION: &str = "CARGO_NEAR_VERSION";
pub(crate) const CARGO_NEAR_ABI_SCHEMA_VERSION: &str = "CARGO_NEAR_ABI_SCHEMA_VERSION";

use std::collections::HashMap;

pub mod nep330 {
    // ====================== NEP-330 1.2.0 - Build Details Extension ===========
    /// NEP-330 1.2.0
    pub const BUILD_ENVIRONMENT: &str = "NEP330_BUILD_INFO_BUILD_ENVIRONMENT";
    /// NEP-330 1.2.0
    pub const BUILD_COMMAND: &str = "NEP330_BUILD_INFO_BUILD_COMMAND";
    /// NEP-330 1.2.0
    pub const CONTRACT_PATH: &str = "NEP330_BUILD_INFO_CONTRACT_PATH";
    /// NEP-330 1.2.0
    pub const SOURCE_CODE_SNAPSHOT: &str = "NEP330_BUILD_INFO_SOURCE_CODE_SNAPSHOT";
    // ====================== End section =======================================

    // ====================== NEP-330 1.1.0 - Contract Metadata Extension ===========
    /// NEP-330 1.1.0
    pub const LINK: &str = "NEP330_LINK";
    /// NEP-330 1.1.0
    pub const VERSION: &str = "NEP330_VERSION";
}
pub const NEP330_OUTPUT_WASM_PATH: &str = "NEP330_BUILD_INFO_OUTPUT_WASM_PATH";

pub fn is_inside_docker_context() -> bool {
    std::env::var(nep330::BUILD_ENVIRONMENT).is_ok()
}

pub fn print_nep330_env() {
    let mut env_map: HashMap<&str, String> = HashMap::new();
    for key in [
        nep330::BUILD_ENVIRONMENT,
        nep330::CONTRACT_PATH,
        nep330::SOURCE_CODE_SNAPSHOT,
        NEP330_OUTPUT_WASM_PATH,
    ] {
        let value = std::env::var(key).unwrap_or("unset".to_string());
        env_map.insert(key, value);
    }
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Variables, relevant for reproducible builds:\n{}",
        crate::pretty_print::indent_payload(&format!("{:#?}", env_map))
    );
}
