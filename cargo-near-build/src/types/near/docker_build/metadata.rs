use colored::Colorize;
use serde::Deserialize;

use serde_json::Value;
use std::{collections::BTreeMap as Map, str::FromStr, thread, time::Duration};

use crate::types::cargo::metadata::CrateMetadata;

#[derive(Deserialize, Debug)]
/// parsed from `[package.metadata.near.reproducible_build]` in Cargo.toml
pub struct ReproducibleBuild {
    image: String,
    image_digest: String,
    pub passed_env: Option<Vec<String>>,
    pub container_build_command: Option<Vec<String>>,
    /// a clonable git remote url,
    /// currently, only ones, starting with `https://`, are supported;
    /// parsed from `package.repository`
    #[serde(skip)]
    pub repository: Option<url::Url>,

    #[serde(flatten)]
    unknown_keys: Map<String, Value>,
}

#[allow(clippy::write_literal)]
impl std::fmt::Display for ReproducibleBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;

        writeln!(f, "    {}: {}", "image", self.image)?;
        writeln!(f, "    {}: {}", "image digest", self.image_digest)?;
        if let Some(ref passed_env) = self.passed_env {
            writeln!(
                f,
                "    {}: {:?}",
                "passed environment variables", passed_env
            )?;
        } else {
            writeln!(
                f,
                "    {}: {}",
                "passed environment variables",
                "ABSENT".green()
            )?;
        }
        if let Some(ref cmd) = self.container_build_command {
            writeln!(f, "    {}: {:?}", "container build command", cmd)?;
        } else {
            writeln!(
                f,
                "    {}: {}",
                "container build command",
                "ABSENT".yellow()
            )?;
        }
        writeln!(
            f,
            "    {}: {}",
            "clonable remote of git repository",
            self.repository
                .clone()
                .map(|url| format!("{}", url))
                .unwrap_or("<empty>".to_string())
        )?;
        Ok(())
    }
}

impl ReproducibleBuild {
    fn validate_image(&self) -> eyre::Result<()> {
        if self
            .image
            .chars()
            .any(|c| !c.is_ascii() || c.is_ascii_control() || c.is_ascii_whitespace())
        {
            return Err(eyre::eyre!(
                "{}: `{}`\n{}",
                "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml",
                self.image,
                "`image`: string contains invalid characters",
            ));
        }
        Ok(())
    }
    fn validate_image_digest(&self) -> eyre::Result<()> {
        if self
            .image_digest
            .chars()
            .any(|c| !c.is_ascii() || c.is_ascii_control() || c.is_ascii_whitespace())
        {
            return Err(eyre::eyre!(
                "{}: `{}`\n{}",
                "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml",
                self.image_digest,
                "`image_digest`: string contains invalid characters",
            ));
        }
        Ok(())
    }
    fn validate_container_build_command(&self) -> eyre::Result<()> {
        let is_cargo_near = {
            let build_command = self.container_build_command.clone().unwrap_or_default();
            Some("cargo") == build_command.first().map(AsRef::as_ref)
                && Some("near") == build_command.get(1).map(AsRef::as_ref)
        };
        for command_token in self.container_build_command.clone().unwrap_or_default() {
            if command_token
                .chars()
                .any(|c| !c.is_ascii() || c.is_ascii_control() || c.is_ascii_whitespace())
            {
                return Err(eyre::eyre!(
                    "{}: `{}`\n{}",
                    "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml",
                    command_token,
                    "`container_build_command`: string token contains invalid characters",
                ));
            }
            if is_cargo_near && command_token == "--no-locked" {
                return Err(eyre::eyre!(
                    "{}:\n{}",
                    "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml",
                    "`container_build_command`: `--no-locked` forbidden for `cargo near` build command",
                ));
            }
        }
        Ok(())
    }

    fn validate_if_unknown_keys_present(&self) -> eyre::Result<()> {
        if !self.unknown_keys.is_empty() {
            let keys = self
                .unknown_keys
                .keys()
                .map(|element| element.as_str())
                .collect::<Vec<_>>();
            return Err(eyre::eyre!(
                "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml, contains unknown keys: `{}`",
                keys.join(",")
            ));
        }
        Ok(())
    }

    fn validate_repository(&self) -> eyre::Result<()> {
        match self.repository {
            Some(ref repository) => {
                if repository.scheme() != "https" {
                    return Err(eyre::eyre!(
                        "{}: {}\n{}",
                        "Malformed NEP330 metadata in Cargo.toml:",
                        repository,
                        "`[package.repository]`: only `https` scheme is supported at the moment",
                    ));
                }
            }
            None => {
                return Err(eyre::eyre!(
                    "{}: \n{}",
                    "Malformed NEP330 metadata in Cargo.toml",
                    "`[package.repository]`: should not be empty",
                ));
            }
        }
        Ok(())
    }

    fn validate(&self) -> eyre::Result<()> {
        self.validate_image()?;
        self.validate_image_digest()?;
        self.validate_container_build_command()?;
        self.validate_if_unknown_keys_present()?;
        self.validate_repository()?;

        if self.passed_env.is_some() && self.container_build_command.is_none() {
            return Err(eyre::eyre!(
                    "{}: \n{}",
                    "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml",
                    "using optional `passed_env` field requires that `container_build_command` is set too",
                ));
        }
        Ok(())
    }
    pub fn parse(cargo_metadata: &CrateMetadata) -> eyre::Result<Self> {
        let build_meta_value = cargo_metadata
            .root_package
            .metadata
            .get("near")
            .and_then(|value| value.get("reproducible_build"));

        let mut build_meta: ReproducibleBuild = match build_meta_value {
            None => {
                println!(
                    "{}{}{}",
                    "An error with missing ".yellow(),
                    "`[package.metadata.near.reproducible_build]`".magenta(),
                    " in Cargo.toml has been encountered...".yellow()
                );
                println!(
                    "{}",
                    "You can choose to disable docker build with `--no-docker` flag...".cyan()
                );
                thread::sleep(Duration::new(7, 0));
                println!();
                println!(
                    "{}{}{}",
                    "Alternatively you can add and commit ".cyan(),
                    "`[package.metadata.near.reproducible_build]` ".magenta(),
                    "to your contract's Cargo.toml:".cyan()
                );
                println!("{}{}", "- default values for the section can be found at ".cyan(), 
                    "https://github.com/near/cargo-near/blob/main/cargo-near/src/commands/new/new-project-template/Cargo.toml.template#L14-L25".magenta());
                println!(
                    "{}{}",
                    "- the same can also be found in Cargo.toml of template project, generated by "
                        .cyan(),
                    "`cargo near new`".magenta()
                );

                thread::sleep(Duration::new(12, 0));

                return Err(eyre::eyre!(
                    "Missing `[package.metadata.near.reproducible_build]` in Cargo.toml"
                ));
            }
            Some(build_meta_value) => {
                serde_json::from_value(build_meta_value.clone()).map_err(|err| {
                    eyre::eyre!(
                        "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml: {}",
                        err
                    )
                })?
            }
        };
        build_meta.repository = cargo_metadata
            .root_package
            .repository
            .clone()
            .map(|url| <url::Url as FromStr>::from_str(&url))
            .transpose()?;
        build_meta.validate()?;
        println!("{} {}", "reproducible build metadata:".green(), build_meta);
        if build_meta.container_build_command.is_some() {
            println!(
                "{}", "using `container_build_command` from `[package.metadata.near.reproducible_build]` in Cargo.toml".cyan()
            );
        }
        Ok(build_meta)
    }
    pub fn concat_image(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.image);
        result.push('@');
        result.push_str(&self.image_digest);
        result
    }
}
