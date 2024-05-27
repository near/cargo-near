use colored::Colorize;
use serde::Deserialize;

use crate::types::metadata::CrateMetadata;
use serde_json::Value;
use std::collections::BTreeMap as Map;

#[derive(Deserialize, Debug)]
pub(super) struct ReproducibleBuild {
    image: String,
    image_digest: String,
    pub container_build_command: Option<String>,
    /// a string, containing https://git-scm.com/docs/git-clone#URLS,
    /// currently, only ones, starting with `https://`, and ending in `.git` are supported
    pub source_code_git_url: url::Url,

    #[serde(flatten)]
    unknown_keys: Map<String, Value>,
}

#[allow(clippy::write_literal)]
impl std::fmt::Display for ReproducibleBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;

        writeln!(f, "    {}: {}", "image", self.image)?;
        writeln!(f, "    {}: {}", "image digest", self.image_digest)?;
        if let Some(ref cmd) = self.container_build_command {
            writeln!(f, "    {}: {}", "container build command", cmd)?;
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
            "source code git url", self.source_code_git_url
        )?;
        Ok(())
    }
}

impl ReproducibleBuild {
    fn validate(&self) -> color_eyre::eyre::Result<()> {
        if !self.unknown_keys.is_empty() {
            let keys = self
                .unknown_keys
                .keys()
                .map(|element| element.as_str())
                .collect::<Vec<_>>();
            return Err(color_eyre::eyre::eyre!(
                "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml, contains unknown keys: `{}`",
                keys.join(",")
            ));
        }
        if self.source_code_git_url.scheme() != "https" {
            return Err(color_eyre::eyre::eyre!(
                "{}: {}\n{}",
                "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml",
                self.source_code_git_url,
                "`source_code_git_url`: only `https` scheme is supported at the moment",
            ));
        }
        if !self.source_code_git_url.path().ends_with(".git") {
            return Err(color_eyre::eyre::eyre!(
                "{}: {}\n{}",
                "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml",
                self.source_code_git_url,
                "`source_code_git_url`: must end with `.git`",
            ));
        }
        Ok(())
    }
    pub(super) fn parse(cargo_metadata: &CrateMetadata) -> color_eyre::eyre::Result<Self> {
        let build_meta_value = cargo_metadata
            .root_package
            .metadata
            .get("near")
            .and_then(|value| value.get("reproducible_build"));

        let build_meta: ReproducibleBuild = match build_meta_value {
            None => {
                return Err(color_eyre::eyre::eyre!(
                    "Missing `[package.metadata.near.reproducible_build]` in Cargo.toml"
                ))
            }
            Some(build_meta_value) => {
                serde_json::from_value(build_meta_value.clone()).map_err(|err| {
                    color_eyre::eyre::eyre!(
                        "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml: {}",
                        err
                    )
                })?
            }
        };
        build_meta.validate()?;
        println!("{} {}", "reproducible build metadata:".green(), build_meta);
        if build_meta.container_build_command.is_some() {
            println!(
                "{}", "using `container_build_command` from `[package.metadata.near.reproducible_build]` in Cargo.toml".cyan()
            );
        }
        Ok(build_meta)
    }
    pub(super) fn concat_image(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.image);
        result.push('@');
        result.push_str(&self.image_digest);
        let result = result
            .chars()
            .filter(|c| c.is_ascii())
            .filter(|c| !c.is_ascii_control())
            .filter(|c| !c.is_ascii_whitespace())
            .collect();
        result
    }
}
