use colored::Colorize;
use serde::Deserialize;

use crate::{types::metadata::CrateMetadata, util};
use serde_json::Value;
use std::collections::BTreeMap as Map;

#[derive(Deserialize, Debug)]
pub(super) struct ReproducibleBuild {
    image: String,
    image_digest: String,
    pub container_build_command: Option<String>,
    /// a string, containing https://git-scm.com/docs/git-clone#URLS,
    /// currently, only ones, starting with `https://`, and ending in `.git` are supported
    pub source_code_git_url: String,

    #[serde(flatten)]
    unknown_keys: Map<String, Value>,
}

impl ReproducibleBuild {
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

        if !build_meta.unknown_keys.is_empty() {
            let keys = build_meta
                .unknown_keys
                .keys()
                .map(|element| element.as_str())
                .collect::<Vec<_>>();
            return Err(color_eyre::eyre::eyre!(
                "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml, contains unknown keys: `{}`",
                keys.join(",")
            ));
        }

        if !build_meta.source_code_git_url.ends_with(".git")
            || !build_meta.source_code_git_url.starts_with("https://")
        {
            return Err(color_eyre::eyre::eyre!(
                "{}: {}\n{}",
                "Malformed `[package.metadata.near.reproducible_build]` in Cargo.toml",
                build_meta.source_code_git_url,
                "`source_code_git_url` should start with `https://` and end with `.git`",
            ));
        }

        println!(
            "{}",
            util::indent_string(&format!(
                "{} {:#?}",
                "reproducible build metadata:".green(),
                build_meta
            ))
        );
        if build_meta.container_build_command.is_some() {
            println!(
                " {}", "using `container_build_command` from `[package.metadata.near.reproducible_build]` in Cargo.toml".cyan()
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
        println!(" {} {}", "docker image to be used:".green(), result,);
        result
    }
}
