use color_eyre::eyre::{ContextCompat, WrapErr};

pub(super) mod dirty;

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

    let repo = git2::Repository::open(contract_path)?;
    let repo_id = repo.revparse_single("HEAD")?.id();

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
