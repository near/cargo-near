use std::time::Duration;

use colored::Colorize;
use eyre::{ContextCompat, WrapErr};

use crate::camino;
use crate::types::near::build::input::BuildContext;
use crate::types::near::docker_build::WARN_BECOMES_ERR;
use serde_json::to_string;

pub fn check_then_handle(
    context: BuildContext,
    repo_root: &camino::Utf8PathBuf,
) -> eyre::Result<()> {
    let result = check(repo_root);
    match (result, context) {
        (Err(err), BuildContext::Deploy) => {
            println!(
                "{}",
                "Either commit and push, or revert following changes to continue deployment:"
                    .yellow()
            );
            Err(err)
        }
        (Err(err), BuildContext::Build) => {
            println!();
            println!("{}: {}", "WARNING".red(), format!("{}", err).yellow());
            std::thread::sleep(Duration::new(3, 0));
            println!();
            println!("{}", WARN_BECOMES_ERR.red(),);
            // this is magic to help user notice:
            std::thread::sleep(Duration::new(5, 0));

            Ok(())
        }
        _ => Ok(()),
    }
}
fn check(repo_root: &camino::Utf8PathBuf) -> eyre::Result<()> {
    let repo = git2::Repository::open(repo_root)?;
    let mut dirty_files = Vec::new();
    // Include each submodule so that the error message can provide
    // specifically *which* files in a submodule are modified.
    status_submodules(&repo, &mut dirty_files)?;

    if dirty_files.is_empty() {
        return Ok(());
    }
    Err(eyre::eyre!(
        "{} files in the working directory contain changes that were \
             not yet committed into git:\n\n{}",
        dirty_files.len(),
        dirty_files
            .iter()
            .map(to_string)
            .collect::<Result<Vec<_>, _>>()
            .wrap_err("Error parsing PathBaf")?
            .join("\n")
    ))
}

// Helper to collect dirty statuses while recursing into submodules.
fn status_submodules(
    repo: &git2::Repository,
    dirty_files: &mut Vec<std::path::PathBuf>,
) -> eyre::Result<()> {
    collect_statuses(repo, dirty_files)?;
    for submodule in repo.submodules()? {
        // Ignore submodules that don't open, they are probably not initialized.
        // If its files are required, then the verification step should fail.
        if let Ok(sub_repo) = submodule.open() {
            status_submodules(&sub_repo, dirty_files)?;
        }
    }
    Ok(())
}

// Helper to collect dirty statuses for a single repo.
fn collect_statuses(
    repo: &git2::Repository,
    dirty_files: &mut Vec<std::path::PathBuf>,
) -> eyre::Result<()> {
    let mut status_opts = git2::StatusOptions::new();
    // Exclude submodules, as they are being handled manually by recursing
    // into each one so that details about specific files can be
    // retrieved.
    status_opts
        .exclude_submodules(true)
        .include_ignored(true)
        .include_untracked(true);
    let repo_statuses = repo.statuses(Some(&mut status_opts)).with_context(|| {
        format!(
            "Failed to retrieve git status from repo {}",
            repo.path().display()
        )
    })?;
    let workdir = repo
        .workdir()
        .wrap_err("no workdir discovered in bare repository")?;
    let this_dirty = repo_statuses.iter().filter_map(|entry| {
        let path = entry.path().expect("valid utf-8 path");
        if entry.status() == git2::Status::IGNORED {
            return None;
        }
        Some(workdir.join(path))
    });
    dirty_files.extend(this_dirty);
    Ok(())
}
