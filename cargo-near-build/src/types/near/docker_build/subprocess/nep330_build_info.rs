use crate::docker::DockerBuildOpts;
use crate::types::near::docker_build::{cloned_repo, metadata};
use eyre::ContextCompat;
use near_verify_rs::types::source_id;

#[derive(Clone, Debug)]
pub struct BuildInfoMixed {
    /// [near_verify_rs::env_keys::BUILD_ENVIRONMENT]
    pub build_environment: String,
    /// [near_verify_rs::env_keys::CONTRACT_PATH]
    pub contract_path: String,
    /// [near_verify_rs::env_keys::SOURCE_CODE_SNAPSHOT]
    pub source_code_snapshot: source_id::SourceId,
    /// [near_verify_rs::env_keys::LINK]
    pub link: Option<String>,
    /// [near_verify_rs::env_keys::BUILD_COMMAND]
    pub build_command: Vec<String>,
}
fn compute_repo_link_hint(
    applied_build_meta: &metadata::AppliedReproducibleBuild,
    cloned_repo: &cloned_repo::ClonedRepo,
) -> Option<String> {
    let repo_link_url = applied_build_meta.repository.clone().expect(
        "expected to be [Option::Some] due to [metadata::ReproducibleBuild::validate_repository] rule"
    );
    let revision = cloned_repo.initial_crate_in_repo.head.to_string();

    if repo_link_url.host_str() == Some("github.com") {
        let existing_path = repo_link_url.path();
        let existing_path = if existing_path.ends_with(".git") {
            existing_path.trim_end_matches(".git")
        } else {
            existing_path
        };

        Some(
            repo_link_url
                .join(&format!("{existing_path}/tree/{revision}"))
                .ok()?
                .to_string(),
        )
    } else {
        Some(repo_link_url.to_string())
    }
}

impl BuildInfoMixed {
    pub fn new(
        opts: &DockerBuildOpts,
        applied_build_meta: &metadata::AppliedReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> eyre::Result<Self> {
        let build_environment = applied_build_meta.concat_image();
        let contract_path = cloned_repo
            .initial_crate_in_repo
            .unix_relative_path()?
            .to_str()
            .wrap_err("non UTF-8 unix path computed as contract path")?
            .to_string();

        let source_code_snapshot = source_id::SourceId::for_git(
            // this unwrap depends on `metadata::ReproducibleBuild::validate` logic
            applied_build_meta.repository.as_ref().unwrap(),
            source_id::GitReference::Rev(cloned_repo.initial_crate_in_repo.head.to_string()),
        )
        .map_err(|err| eyre::eyre!("compute SourceId {}", err))?;

        let link = compute_repo_link_hint(applied_build_meta, cloned_repo);
        let build_command = opts.get_cli_build_command_in_docker(applied_build_meta)?;

        Ok(Self {
            build_environment,
            contract_path,
            source_code_snapshot,
            link,
            build_command,
        })
    }
}

impl From<BuildInfoMixed>
    for near_verify_rs::types::contract_source_metadata::ContractSourceMetadata
{
    fn from(
        value: BuildInfoMixed,
    ) -> near_verify_rs::types::contract_source_metadata::ContractSourceMetadata {
        near_verify_rs::types::contract_source_metadata::ContractSourceMetadata {
            version: None,
            link: value.link,
            standards: vec![],
            build_info: Some(near_verify_rs::types::contract_source_metadata::BuildInfo {
                build_command: value.build_command,
                build_environment: value.build_environment,
                source_code_snapshot: value.source_code_snapshot.as_url().to_string(),
                contract_path: value.contract_path,
                output_wasm_path: None,
            }),
        }
    }
}
