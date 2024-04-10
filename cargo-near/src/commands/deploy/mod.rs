use color_eyre::eyre::{ContextCompat, WrapErr};
use serde_json::to_string;

use near_cli_rs::commands::contract::deploy::initialize_mode::InitializeMode;

use crate::commands::build_command;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    #[interactive_clap(flatten)]
    /// Specify a build command args:
    build_command_args: build_command::BuildCommand,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    contract_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    initialize: InitializeMode,
}

#[derive(Debug, Clone)]
pub struct ContractContext(near_cli_rs::commands::contract::deploy::ContractFileContext);

impl ContractContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let contract_path: camino::Utf8PathBuf =
            if let Some(manifest_path) = &scope.build_command_args.manifest_path {
                manifest_path.into()
            } else {
                camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?).map_err(|err| {
                    color_eyre::eyre::eyre!("Failed to convert path {}", err.to_string_lossy())
                })?
            };

        let file_path = if !scope.build_command_args.no_docker {
            // TODO: clone to tmp folder and checkout specific revision must be separate steps
            eprintln!(
                "\n The URL of the remote repository:\n {}\n",
                remote_repo_url(&contract_path)?
            );
            build_command::docker_run(scope.build_command_args.clone())?
        } else {
            build_command::build::run(scope.build_command_args.clone())?.path
        };

        Ok(Self(
            near_cli_rs::commands::contract::deploy::ContractFileContext {
                global_context: previous_context,
                receiver_account_id: scope.contract_account_id.clone().into(),
                signer_account_id: scope.contract_account_id.clone().into(),
                code: std::fs::read(file_path)?,
            },
        ))
    }
}

impl From<ContractContext> for near_cli_rs::commands::contract::deploy::ContractFileContext {
    fn from(item: ContractContext) -> Self {
        item.0
    }
}

impl interactive_clap::FromCli for Contract {
    type FromCliContext = near_cli_rs::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        let build_command_args =
            if let Some(cli_build_command_args) = &clap_variant.build_command_args {
                build_command::BuildCommand {
                    no_docker: cli_build_command_args.no_docker,
                    no_release: cli_build_command_args.no_release,
                    no_abi: cli_build_command_args.no_abi,
                    no_embed_abi: cli_build_command_args.no_embed_abi,
                    no_doc: cli_build_command_args.no_doc,
                    out_dir: cli_build_command_args.out_dir.clone(),
                    manifest_path: cli_build_command_args.manifest_path.clone(),
                    color: cli_build_command_args.color.clone(),
                }
            } else {
                build_command::BuildCommand::default()
            };

        if clap_variant.contract_account_id.is_none() {
            clap_variant.contract_account_id = match Self::input_contract_account_id(&context) {
                Ok(Some(contract_account_id)) => Some(contract_account_id),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let contract_account_id = clap_variant
            .contract_account_id
            .clone()
            .expect("Unexpected error");

        let new_context_scope = InteractiveClapContextScopeForContract {
            build_command_args,
            contract_account_id,
        };

        let output_context =
            match ContractContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };

        match InitializeMode::from_cli(clap_variant.initialize.take(), output_context.into()) {
            interactive_clap::ResultFromCli::Ok(initialize) => {
                clap_variant.initialize = Some(initialize);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_initialize) => {
                clap_variant.initialize = optional_initialize;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_initialize, err) => {
                clap_variant.initialize = optional_initialize;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

impl Contract {
    pub fn input_contract_account_id(
        context: &near_cli_rs::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        near_cli_rs::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
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

fn remote_repo_url(contract_path: &camino::Utf8PathBuf) -> color_eyre::Result<reqwest::Url> {
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
