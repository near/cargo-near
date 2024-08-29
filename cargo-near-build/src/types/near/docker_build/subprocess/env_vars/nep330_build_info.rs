use crate::types::near::docker_build::{cloned_repo, metadata};
use crate::{env_keys, types::source_id};
use eyre::ContextCompat;

pub(super) struct BuildInfo {
    build_environment: String,
    contract_path: String,
    source_code_snapshot: source_id::SourceId,
}

impl BuildInfo {
    pub fn new(
        docker_build_meta: &metadata::ReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> eyre::Result<Self> {
        let build_environment = docker_build_meta.concat_image();
        let contract_path = cloned_repo
            .initial_crate_in_repo
            .unix_relative_path()?
            .to_str()
            .wrap_err("non UTF-8 unix path computed as contract path")?
            .to_string();

        let source_code_snapshot = source_id::SourceId::for_git(
            // this unwrap depends on `metadata::ReproducibleBuild::validate` logic
            docker_build_meta.repository.as_ref().unwrap(),
            source_id::GitReference::Rev(cloned_repo.initial_crate_in_repo.head.to_string()),
        )
        .map_err(|err| eyre::eyre!("compute SourceId {}", err))?;
        Ok(Self {
            build_environment,
            contract_path,
            source_code_snapshot,
        })
    }

    pub fn docker_args(&self) -> Vec<String> {
        let mut result = vec![
            "--env".to_string(),
            format!(
                "{}={}",
                env_keys::nep330::BUILD_ENVIRONMENT,
                self.build_environment
            ),
            "--env".to_string(),
            format!(
                "{}={}",
                env_keys::nep330::SOURCE_CODE_SNAPSHOT,
                self.source_code_snapshot.as_url()
            ),
        ];

        result.extend(vec![
            "--env".to_string(),
            format!("{}={}", env_keys::nep330::CONTRACT_PATH, self.contract_path),
        ]);

        result
    }
}
