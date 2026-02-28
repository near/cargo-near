use colored::Colorize;

#[cfg(windows)]
const BIN_NAME: &str = "cargo-near.exe";
#[cfg(not(windows))]
const BIN_NAME: &str = "cargo-near";

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = SelfUpdateCommandContext)]
pub struct SelfUpdateCommand;

#[derive(Debug, Clone)]
pub struct SelfUpdateCommandContext;

impl SelfUpdateCommandContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        _scope: &<SelfUpdateCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let Ok(latest_version) = get_latest_version() else {
            eprintln!("Failed to get the latest version of cargo-near");
            return Ok(Self);
        };

        if let Ok(current_version) = semver::Version::parse(self_update::cargo_crate_version!())
            && let Ok(latest_version) = semver::Version::parse(&latest_version)
            && current_version >= latest_version
        {
            println!(
                "cargo-near is up to date ({})\n",
                format!("v{}", self_update::cargo_crate_version!()).green()
            );
            return Ok(Self);
        }

        self_update::backends::github::Update::configure()
            .repo_owner("near")
            .repo_name("cargo-near")
            .bin_path_in_archive(
                format!("cargo-near-{}/{}", self_update::get_target(), BIN_NAME).as_str(),
            )
            .bin_name(BIN_NAME)
            .show_download_progress(true)
            .current_version(self_update::cargo_crate_version!())
            .target_version_tag(&format!("cargo-near-v{latest_version}"))
            .build()
            .map_err(|err| color_eyre::eyre::eyre!("Failed to build self_update: {err}"))?
            .update()
            .map_err(|err| color_eyre::eyre::eyre!("Failed to update cargo-near: {err}"))?;

        Ok(Self)
    }
}

pub fn get_latest_version() -> color_eyre::eyre::Result<String> {
    let release = self_update::backends::github::Update::configure()
        .repo_owner("near")
        .repo_name("cargo-near")
        .bin_name(BIN_NAME)
        .current_version(self_update::cargo_crate_version!())
        .build()
        .map_err(|err| color_eyre::eyre::eyre!("Failed to build self_update: {err}"))?
        .get_latest_release()
        .map_err(|err| color_eyre::eyre::eyre!("Failed to get latest release: {err}"))?;

    Ok(release
        .version
        .strip_prefix("cargo-near-v")
        .unwrap_or(&release.version)
        .to_string())
}
