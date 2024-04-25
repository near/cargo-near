use colored::Colorize;

const BETWEEN_ATTEMPTS_SLEEP: std::time::Duration = std::time::Duration::from_millis(100);

pub fn check(git_url: &str, commit_id: git2::Oid) -> color_eyre::Result<()> {
    for attempt in 1..=5 {
        let tmp_clone_destination = tempfile::tempdir()?;
        println!(
            " {} `{}` -> `{:?}`",
            format!("Clone attempt {}:", attempt).green(),
            git_url,
            tmp_clone_destination
        );
        let repo = git2::Repository::clone_recurse(git_url, tmp_clone_destination.path());

        match repo {
            Ok(repo) => {
                println!(" {}", "Checking if HEAD is present...".green());
                repo.find_commit(commit_id)?;
                println!(
                    " {} {} in `{}` -> `{}`",
                    "commit was found in repo:".green(),
                    commit_id,
                    git_url,
                    repo.path().display(),
                );
                return Ok(());
            }
            Err(err) => {
                println!(" {} {:?}", "Encountered error:".yellow(), err,);
                std::thread::sleep(BETWEEN_ATTEMPTS_SLEEP);
            }
        }
    }

    Err(color_eyre::eyre::eyre!(
        "Failed to verify that HEAD was pushed by cloning {}. Exceeded max attempts",
        git_url
    ))
}
