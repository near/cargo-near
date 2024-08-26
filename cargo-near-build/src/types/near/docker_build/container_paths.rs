use super::cloned_repo;
use eyre::ContextCompat;

pub struct Paths {
    pub host_volume_arg: String,
    pub crate_path: String,
}

const NEP330_REPO_MOUNT: &str = "/home/near/code";

impl Paths {
    pub fn compute(cloned_repo: &cloned_repo::ClonedRepo) -> eyre::Result<Self> {
        let mounted_repo = NEP330_REPO_MOUNT.to_string();
        let host_volume_arg = format!(
            "{}:{}",
            cloned_repo.tmp_repo_dir.path().to_string_lossy(),
            &mounted_repo
        );
        let crate_path = {
            let mut repo_path = unix_path::Path::new(NEP330_REPO_MOUNT).to_path_buf();
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
