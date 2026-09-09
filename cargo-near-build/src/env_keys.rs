/// this is [`CARGO_TARGET_DIR`](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
pub const CARGO_TARGET_DIR: &str = "CARGO_TARGET_DIR";
/// this variable is set to `"true"` during ABI generation operation  
pub const BUILD_RS_ABI_STEP_HINT: &str = "CARGO_NEAR_ABI_GENERATION";

/// <https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags>
///
/// this behaviour that
/// 1. for the wasm build stage, the canonical carrier is [`CARGO_ENCODED_RUSTFLAGS`]
///    (more robust than RUSTFLAGS against args containing spaces). RUSTFLAGS supplied via
///    `--env` is still honored — it gets translated into the encoded form, and
///    CARGO_ENCODED_RUSTFLAGS-via-`--env` wins over RUSTFLAGS-via-`--env` (matching cargo's
///    own precedence)
/// 2. the default token list is `-C link-arg=-s`
/// 3. `--cfg near` is force-appended to the effective tokens (after any override) so
///    `near-sdk` selects the on-chain host-function path; it cannot be dropped by user override
/// 4. for the abi gen stage, RUSTFLAGS is still used with `-Awarnings`
/// 5. ambient RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS from the user's shell are NOT inherited
///
/// is documented in RUSTFLAGS section of README.md
pub const RUSTFLAGS: &str = "RUSTFLAGS";

/// <https://rust-lang.github.io/rustup/environment-variables.html>
pub const RUSTUP_TOOLCHAIN: &str = "RUSTUP_TOOLCHAIN";

/// <https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags>
///
/// See the doc on [`RUSTFLAGS`] for the full behaviour. Summary: this is the canonical carrier
/// for wasm-build rustflags (set explicitly by cargo-near, 0x1f-separated), ambient values from
/// the user's shell are stripped, and `--cfg near` is force-appended.
pub const CARGO_ENCODED_RUSTFLAGS: &str = "CARGO_ENCODED_RUSTFLAGS";

/// see `PROFILE` in <https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts>
pub const RUST_PROFILE: &str = "PROFILE";

#[cfg(feature = "build_internal")]
pub(crate) const CARGO_NEAR_ABI_PATH: &str = "CARGO_NEAR_ABI_PATH";

#[cfg(feature = "build_internal")]
pub(crate) const CARGO_NEAR_VERSION: &str = "CARGO_NEAR_VERSION";
#[cfg(feature = "build_internal")]
pub(crate) const CARGO_NEAR_ABI_SCHEMA_VERSION: &str = "CARGO_NEAR_ABI_SCHEMA_VERSION";

pub const COLOR_PREFERENCE_NO_COLOR: &str = "NO_COLOR";

use std::collections::HashMap;

pub mod nep330 {
    // ====================== NEP-330 1.2.0 - Build Details Extension ===========
    /// NEP-330 1.3.0
    pub const OUTPUT_WASM_PATH: &str = "NEP330_BUILD_INFO_OUTPUT_WASM_PATH";
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

    /// NEP-330 1.2.0
    pub const NEP330_REPO_MOUNT: &str = "/home/near/code";
}

pub fn is_inside_docker_context() -> bool {
    std::env::var(nep330::BUILD_ENVIRONMENT).is_ok()
}

pub fn print_nep330_env() {
    let mut env_map: HashMap<&str, String> = HashMap::new();
    for key in [
        nep330::BUILD_ENVIRONMENT,
        nep330::CONTRACT_PATH,
        nep330::SOURCE_CODE_SNAPSHOT,
        nep330::LINK,
    ] {
        let value = std::env::var(key).unwrap_or("unset".to_string());
        env_map.insert(key, value);
    }
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Variables, relevant for reproducible builds:\n{}",
        crate::pretty_print::indent_payload(&format!("{env_map:#?}"))
    );
}
