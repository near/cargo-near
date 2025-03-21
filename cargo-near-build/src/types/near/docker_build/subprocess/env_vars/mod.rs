pub mod nep330_build_info;

const RUST_LOG_EXPORT: &str = "RUST_LOG=info";
use nep330_build_info::BuildInfoMixed;

use crate::types::near::docker_build::{cloned_repo, metadata};

pub struct EnvVars {
    build_info: BuildInfoMixed,
    rust_log: String,
}

impl EnvVars {
    pub fn new(
        docker_build_meta: &metadata::ReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> eyre::Result<Self> {
        let build_info = BuildInfoMixed::new(docker_build_meta, cloned_repo)?;
        // this unwrap depends on `metadata::ReproducibleBuild::validate` logic
        Ok(Self {
            build_info,
            rust_log: RUST_LOG_EXPORT.to_string(),
        })
    }

    pub fn docker_args(&self) -> Vec<String> {
        let mut result = self.build_info.docker_args();

        result.extend(vec!["--env".to_string(), self.rust_log.clone()]);
        result
    }
}
