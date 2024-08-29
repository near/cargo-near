use colored::Colorize;
use eyre::ContextCompat;

#[derive(Debug, Clone)]
pub struct Crate {
    pub repo_root: camino::Utf8PathBuf,
    /// crate path, which is a child of `repo_root`
    pub crate_root: camino::Utf8PathBuf,
    /// HEAD commit OID
    pub head: git2::Oid,
}

impl Crate {
    pub fn find(initial_crate_root: &camino::Utf8PathBuf) -> eyre::Result<Self> {
        let mut search_from = initial_crate_root.clone();

        let mut repo: Option<git2::Repository> = None;

        // NOTE: this cycle logic is needed to detect top level repo, when called from a
        // current dir within a submodule
        while let Ok(repo_root) = git2::Repository::discover_path(&search_from, home::home_dir()) {
            repo = Some(git2::Repository::open(&repo_root)?);
            let workdir = repo
                .as_ref()
                .unwrap()
                .workdir()
                .wrap_err("bare repository has no workdir")?;
            let workdir: camino::Utf8PathBuf = workdir.to_path_buf().try_into()?;
            search_from = match workdir.parent() {
                Some(parent) => parent.to_path_buf(),
                None => break,
            };
        }
        match repo {
            None => Err(eyre::eyre!(
                "Repo containing {} not found",
                initial_crate_root
            )),
            Some(repo) => {
                let workdir = repo.workdir().wrap_err("bare repository has no workdir")?;

                let head = repo.revparse_single("HEAD")?.id();
                println!(
                    "{} {:?}",
                    format!("current HEAD ({}):", repo.path().display()).green(),
                    head
                );
                let result = Crate {
                    repo_root: workdir.to_path_buf().try_into()?,
                    crate_root: initial_crate_root.clone(),
                    head,
                };
                log::debug!("crate in repo: {:?}", result);
                Ok(result)
            }
        }
    }
    pub fn host_relative_path(&self) -> eyre::Result<camino::Utf8PathBuf> {
        pathdiff::diff_utf8_paths(&self.crate_root, &self.repo_root)
            .wrap_err("cannot compute crate's relative path in repo")
    }
    pub fn unix_relative_path(&self) -> eyre::Result<unix_path::PathBuf> {
        let host_relative: camino::Utf8PathBuf = self.host_relative_path()?;

        let path_buf = {
            let iter = host_relative
                .components()
                .map(|component| component.as_str());
            unix_path::PathBuf::from_iter(iter.map(unix_path::Path::new))
        };

        if !path_buf.as_path().is_relative() {
            return Err(eyre::eyre!(
                "crate's path in repo, expressed as a unix path, isn't relative : {:?}",
                path_buf.to_str()
            ));
        }

        Ok(path_buf)
    }
}
