pub mod nep330_build_info;

const RUST_LOG_EXPORT: &str = "RUST_LOG=info";
use nep330_build_info::BuildInfoMixed;

pub struct EnvVars {
    build_info_mixed: BuildInfoMixed,
    rust_log: String,
}

/// TODO #G: move out this type to [near_verify_rs::types::internal] with pub(crate) visibility
impl EnvVars {
    pub fn new(build_info_mixed: BuildInfoMixed) -> eyre::Result<Self> {
        // this unwrap depends on `metadata::ReproducibleBuild::validate` logic
        Ok(Self {
            build_info_mixed,
            rust_log: RUST_LOG_EXPORT.to_string(),
        })
    }

    /// TODO #F3: replace with `additional_docker_args` parameter usage
    pub fn docker_args(&self) -> Vec<String> {
        let mut result = self.build_info_mixed.docker_args();

        result.extend(vec!["--env".to_string(), self.rust_log.clone()]);
        result
    }
}
