use color_eyre::eyre::{ContextCompat, WrapErr};
use serde_json::to_string;

pub(super) fn remote_repo_url(
    contract_path: &camino::Utf8PathBuf,
) -> color_eyre::Result<reqwest::Url> {
    let mut path_cargo_toml = contract_path.clone();
    path_cargo_toml.push("Cargo.toml");
    let cargo_toml = cargo_toml::Manifest::from_slice(
        &std::fs::read(&path_cargo_toml)
            .wrap_err_with(|| format!("Failed to read file <{path_cargo_toml}>"))?,
    )
    .wrap_err("Could not parse 'Cargo.toml'")?;

    let mut remote_repo_url = reqwest::Url::parse(
        cargo_toml
            .package()
            .repository()
            .wrap_err("No reference to the remote repository for this contract was found in the file 'Cargo.toml'.\
                        \nAdd the value 'repository' to the '[package]' section  to continue deployment.")?
    )?;

    let path = remote_repo_url.path().trim_end_matches('/');

    let repo_id = check_repo_state(contract_path)?.to_string();

    let commit = format!("{path}/commit/{repo_id}");

    remote_repo_url.set_path(&commit);
    log::info!("checking existence of {}", remote_repo_url);

    let mut retries_left = (0..5).rev();
    loop {
        let response = reqwest::blocking::get(remote_repo_url.clone())?;

        if retries_left.next().is_none() {
            color_eyre::eyre::bail!("Currently, it is not possible to check for remote repository <{remote_repo_url}>. Try again after a while.")
        }

        // Check if status is within 100-199.
        if response.status().is_informational() {
            eprintln!("Transport error.\nPlease wait. The next try to send this query is happening right now ...");
        }

        // Check if status is within 200-299.
        if response.status().is_success() {
            return Ok(remote_repo_url);
        }

        // Check if status is within 300-399.
        if response.status().is_redirection() {
            return Ok(remote_repo_url);
        }

        // Check if status is within 400-499.
        if response.status().is_client_error() {
            color_eyre::eyre::bail!("Remote repository <{remote_repo_url}> does not exist.")
        }

        // Check if status is within 500-599.
        if response.status().is_server_error() {
            eprintln!("Transport error.\nPlease wait. The next try to send this query is happening right now ...");
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn check_repo_state(contract_path: &camino::Utf8PathBuf) -> color_eyre::Result<git2::Oid> {
    let repo = git2::Repository::open(contract_path)?;
    let mut dirty_files = Vec::new();
    collect_statuses(&repo, &mut dirty_files)?;
    // Include each submodule so that the error message can provide
    // specifically *which* files in a submodule are modified.
    status_submodules(&repo, &mut dirty_files)?;

    if dirty_files.is_empty() {
        return Ok(repo.revparse_single("HEAD")?.id());
    }
    color_eyre::eyre::bail!(
        "{} files in the working directory contain changes that were \
             not yet committed into git:\n\n{}\n\n\
             commit these changes to continue deployment",
        dirty_files.len(),
        dirty_files
            .iter()
            .map(to_string)
            .collect::<Result<Vec<_>, _>>()
            .wrap_err("Error parsing PathBaf")?
            .join("\n")
    )
}

// Helper to collect dirty statuses for a single repo.
fn collect_statuses(
    repo: &git2::Repository,
    dirty_files: &mut Vec<std::path::PathBuf>,
) -> near_cli_rs::CliResult {
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
    let workdir = repo.workdir().unwrap();
    let this_dirty = repo_statuses.iter().filter_map(|entry| {
        let path = entry.path().expect("valid utf-8 path");
        if path.ends_with("Cargo.lock") || entry.status() == git2::Status::IGNORED {
            return None;
        }
        Some(workdir.join(path))
    });
    dirty_files.extend(this_dirty);
    Ok(())
}

// Helper to collect dirty statuses while recursing into submodules.
fn status_submodules(
    repo: &git2::Repository,
    dirty_files: &mut Vec<std::path::PathBuf>,
) -> near_cli_rs::CliResult {
    for submodule in repo.submodules()? {
        // Ignore submodules that don't open, they are probably not initialized.
        // If its files are required, then the verification step should fail.
        if let Ok(sub_repo) = submodule.open() {
            status_submodules(&sub_repo, dirty_files)?;
            collect_statuses(&sub_repo, dirty_files)?;
        }
    }
    Ok(())
}
