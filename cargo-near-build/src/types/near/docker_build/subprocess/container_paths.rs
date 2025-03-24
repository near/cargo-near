use crate::types::near::docker_build::cloned_repo;
use eyre::ContextCompat;

pub struct Paths {
    pub host_volume_arg: String,
    pub crate_path: String,
}

const NEP330_REPO_MOUNT: &str = "/home/near/code";

/// TODO: #E6 remove this redundant method
#[allow(unused)]
fn nep330_contract_path_in_other_spot(
    cloned_repo: &cloned_repo::ClonedRepo,
) -> eyre::Result<String> {
    let contract_path = cloned_repo
        .initial_crate_in_repo
        .unix_relative_path()?
        .to_str()
        .wrap_err("non UTF-8 unix path computed as contract path")?
        .to_string();

    Ok(contract_path)
}

/// TODO: #E5 pass in [crate::types::near::docker_build::subprocess::env_vars::nep330_build_info::BuildInfoMixed::contract_path]
/// TODO: #E6 perform a computation of relative path from [crate::env_keys::nep330::CONTRACT_PATH]
impl Paths {
    pub fn compute(
        cloned_repo: &cloned_repo::ClonedRepo,
        contract_source_workdir: camino::Utf8PathBuf,
    ) -> eyre::Result<Self> {
        let mounted_repo = NEP330_REPO_MOUNT.to_string();
        let host_volume_arg = format!("{}:{}", contract_source_workdir.as_str(), &mounted_repo);
        let crate_path = {
            let mut repo_path = unix_path::Path::new(NEP330_REPO_MOUNT).to_path_buf();
            #[allow(unused_doc_comments)]
            /// TODO #E6 relative_path = unix_path::PathBuf::from_str("path/babe")?;
            repo_path.push(cloned_repo.initial_crate_in_repo.unix_relative_path()?);

            repo_path
                .to_str()
                .wrap_err("non UTF-8 unix path computed as crate path")?
                .to_string()
        };
        Ok(Self {
            host_volume_arg,
            crate_path,
        })
    }
}
