pub mod nep330_build_info;

const RUST_LOG_EXPORT: &str = "RUST_LOG=cargo_near=info";
use nep330_build_info::BuildInfo;

use crate::env_keys;

use super::{cloned_repo, metadata};

pub struct EnvVars {
    build_info: BuildInfo,
    rust_log: String,
    repo_link: url::Url,
    revision: String,
}

impl EnvVars {
    pub fn new(
        docker_build_meta: &metadata::ReproducibleBuild,
        cloned_repo: &cloned_repo::ClonedRepo,
    ) -> eyre::Result<Self> {
        let build_info = BuildInfo::new(docker_build_meta, cloned_repo)?;
        // this unwrap depends on `metadata::ReproducibleBuild::validate` logic
        let repo_link = docker_build_meta.repository.clone().unwrap();
        let revision = cloned_repo.initial_crate_in_repo.head.to_string();
        Ok(Self {
            build_info,
            rust_log: RUST_LOG_EXPORT.to_string(),
            repo_link,
            revision,
        })
    }

    pub fn docker_args(&self) -> Vec<String> {
        let mut result = self.build_info.docker_args();

        if let Some(repo_link_hint) = self.compute_repo_link_hint() {
            result.extend(vec![
                "--env".to_string(),
                format!("{}={}", env_keys::nep330::LINK, repo_link_hint,),
            ]);
        }
        result.extend(vec!["--env".to_string(), self.rust_log.clone()]);
        result
    }
    fn compute_repo_link_hint(&self) -> Option<String> {
        let url = self.repo_link.clone();

        if url.host_str() == Some("github.com") {
            let existing_path = url.path();
            let existing_path = if existing_path.ends_with(".git") {
                existing_path.trim_end_matches(".git")
            } else {
                existing_path
            };

            Some(
                url.join(&format!("{}/tree/{}", existing_path, self.revision))
                    .ok()?
                    .to_string(),
            )
        } else {
            None
        }
    }
}
