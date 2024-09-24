/// this variable is set to `"true"` during ABI generation operation  
pub const BUILD_RS_ABI_STEP_HINT: &str = "CARGO_NEAR_ABI_GENERATION";

pub(crate) const CARGO_NEAR_ABI_PATH: &str = "CARGO_NEAR_ABI_PATH";

pub(crate) const CARGO_NEAR_VERSION: &str = "CARGO_NEAR_VERSION";
pub(crate) const CARGO_NEAR_ABI_SCHEMA_VERSION: &str = "CARGO_NEAR_ABI_SCHEMA_VERSION";

/// module contains variables, which are set to configure build with WASM reproducibility,
/// which correspond to some fields of `ContractSourceMetadata` in <https://github.com/near/NEPs/blob/master/neps/nep-0330.md>
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
    // ====================== End section =======================================
    #[cfg(feature = "docker")]
    pub(crate) mod nonspec {
        pub const SERVER_DISABLE_INTERACTIVE: &str = "CARGO_NEAR_SERVER_BUILD_DISABLE_INTERACTIVE";
    }

    pub(crate) fn print_env() {
        tracing::info!("Variables, relevant for reproducible builds:");
        for key in [BUILD_ENVIRONMENT, CONTRACT_PATH, SOURCE_CODE_SNAPSHOT] {
            let value = std::env::var(key)
                .map(|val| format!("'{}'", val))
                .unwrap_or("unset".to_string());
            tracing::info!("{}={}", key, value);
        }
    }
}
