/// additional argument of [build](crate::build) function, wrapped in an `Option`
#[derive(Debug, Clone)]
pub struct Opts {
    /// override value of [crate::env_keys::nep330::CONTRACT_PATH] environment variable
    pub nep330_contract_path: Option<String>,
    /// override value of [crate::env_keys::nep330::CARGO_TARGET_DIR] environment variable,
    /// which is required to avoid deadlock <https://github.com/rust-lang/cargo/issues/8938> in context of nested (cargo) build
    /// in build-script;
    ///
    /// should best be a subfolder of [crate::env_keys::nep330::CARGO_TARGET_DIR]
    /// of crate being built to work normally
    pub cargo_target_dir: Option<String>,
}
