use crate::docker::DockerBuildOpts;
use crate::env_keys;
use crate::types::near::docker_build::{cloned_repo, metadata};
use eyre::ContextCompat;
use near_verify_rs::types::source_id;

#[derive(Clone, Debug)]
pub struct BuildInfoMixed {
    /// [env_keys::nep330::BUILD_ENVIRONMENT]
    pub build_environment: String,
    /// [env_keys::nep330::CONTRACT_PATH]
    pub contract_path: String,
    /// [env_keys::nep330::SOURCE_CODE_SNAPSHOT]
    pub source_code_snapshot: source_id::SourceId,
    /// [env_keys::nep330::LINK]
    pub link: Option<String>,
    /// [env_keys::nep330::BUILD_COMMAND]
    pub build_command: Vec<String>,
    /// [env_keys::nep330::VERSION]
    pub version: Option<String>,
}
fn compute_repo_link_hint(
    docker_build_meta: &metadata::ReproducibleBuild,
    cloned_repo: &cloned_repo::ClonedRepo,
) -> Option<String> {
    let repo_link_url = docker_build_meta.repository.clone().expect(
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
                .join(&format!("{}/tree/{}", existing_path, revision))
                .ok()?
                .to_string(),
        )
    } else {
        Some(repo_link_url.to_string())
    }
}

impl BuildInfoMixed {
    pub fn new(
        opts: DockerBuildOpts,
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

        let link = compute_repo_link_hint(docker_build_meta, cloned_repo);
        let build_command = opts.get_cli_build_command_in_docker(&docker_build_meta)?;
        let version = Some(
            cloned_repo
                .crate_metadata()
                .root_package
                .version
                .to_string(),
        );
        Ok(Self {
            build_environment,
            contract_path,
            source_code_snapshot,
            link,
            build_command,
            version,
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
        if let Some(ref repo_link_hint) = self.link {
            result.extend(vec![
                "--env".to_string(),
                format!("{}={}", env_keys::nep330::LINK, repo_link_hint),
            ]);
        }
        if let Some(ref version) = self.version {
            result.extend(vec![
                "--env".to_string(),
                format!("{}={}", env_keys::nep330::VERSION, version),
            ]);
        }

        result
    }
}
