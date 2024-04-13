use crate::commands::build_command::BuildCommand;

pub(super) struct ClonedRepo {
    pub tmp_repo: git2::Repository,
    pub tmp_contract_path: std::path::PathBuf,
    pub contract_path: camino::Utf8PathBuf,
    #[allow(unused)]
    tmp_contract_dir: tempfile::TempDir,
}

impl ClonedRepo {
    pub(super) fn clone(args: &BuildCommand) -> color_eyre::eyre::Result<Self> {
        let contract_path: camino::Utf8PathBuf = args.contract_path()?;
        log::info!("ClonedRepo.contract_path: {:?}", contract_path,);

        let tmp_contract_dir = tempfile::tempdir()?;
        let tmp_contract_path = tmp_contract_dir.path().to_path_buf();
        log::info!("ClonedRepo.tmp_contract_path: {:?}", tmp_contract_path);
        let tmp_repo = git2::Repository::clone(contract_path.as_str(), &tmp_contract_path)?;
        Ok(ClonedRepo {
            tmp_repo,
            tmp_contract_path,
            tmp_contract_dir,
            contract_path,
        })
    }
}
