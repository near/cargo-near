use std::str::FromStr;

use eyre::ContextCompat;

use super::nep330_build_info::BuildInfoMixed;

pub struct Paths {
    pub host_volume_arg: String,
    pub crate_path: String,
}

const NEP330_REPO_MOUNT: &str = "/home/near/code";

impl Paths {
    pub fn compute(
        build_info_mixed: &BuildInfoMixed,
        contract_source_workdir: camino::Utf8PathBuf,
    ) -> eyre::Result<Self> {
        let mounted_repo = NEP330_REPO_MOUNT.to_string();
        let host_volume_arg = format!("{}:{}", contract_source_workdir.as_str(), &mounted_repo);
        let crate_path = {
            let mut repo_path = unix_path::Path::new(NEP330_REPO_MOUNT).to_path_buf();
            let relative_crate_path =
                unix_path::PathBuf::from_str(&build_info_mixed.contract_path)?;
            repo_path.push(relative_crate_path);

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
