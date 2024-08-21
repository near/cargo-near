pub const BUILD_RS_ABI_STEP_HINT: &str = "CARGO_NEAR_ABI_GENERATION";

pub const CARGO_NEAR_VERSION: &str = "CARGO_NEAR_VERSION";
pub const CARGO_NEAR_ABI_SCHEMA_VERSION: &str = "CARGO_NEAR_ABI_SCHEMA_VERSION";

pub mod nep330 {

    // ====================== NEP-330 1.2.0 - Build Details Extension ===========
    pub const BUILD_ENVIRONMENT: &str = "NEP330_BUILD_INFO_BUILD_ENVIRONMENT";
    pub const BUILD_COMMAND: &str = "NEP330_BUILD_INFO_BUILD_COMMAND";
    pub const CONTRACT_PATH: &str = "NEP330_BUILD_INFO_CONTRACT_PATH";
    pub const SOURCE_CODE_SNAPSHOT: &str = "NEP330_BUILD_INFO_SOURCE_CODE_SNAPSHOT";
    // ====================== End section =======================================

    // ====================== NEP-330 1.1.0 - Contract Metadata Extension ===========
    pub const LINK: &str = "NEP330_LINK";
    pub const VERSION: &str = "NEP330_VERSION";
    // ====================== End section =======================================
    pub mod nonspec {
        pub const SERVER_DISABLE_INTERACTIVE: &str = "CARGO_NEAR_SERVER_BUILD_DISABLE_INTERACTIVE";
    }

    pub fn print_env() {
        log::info!("Variables, relevant for reproducible builds:");
        for key in [
            BUILD_ENVIRONMENT,
            BUILD_COMMAND,
            CONTRACT_PATH,
            SOURCE_CODE_SNAPSHOT,
        ] {
            let value = std::env::var(key)
                .map(|val| format!("'{}'", val))
                .unwrap_or("unset".to_string());
            log::info!("{}={}", key, value);
        }
    }
}
