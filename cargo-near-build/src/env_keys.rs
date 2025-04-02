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

pub use near_verify_rs::env_keys as nep330;

pub fn is_inside_docker_context() -> bool {
    std::env::var(nep330::BUILD_ENVIRONMENT).is_ok()
}

use near_verify_rs::env_keys::{BUILD_ENVIRONMENT, CONTRACT_PATH, SOURCE_CODE_SNAPSHOT};

pub fn print_nep330_env() {
    let mut env_map: HashMap<&str, String> = HashMap::new();
    for key in [BUILD_ENVIRONMENT, CONTRACT_PATH, SOURCE_CODE_SNAPSHOT] {
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
