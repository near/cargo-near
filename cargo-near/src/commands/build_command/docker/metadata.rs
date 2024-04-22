use colored::Colorize;
use serde::Deserialize;

use crate::{types::metadata::CrateMetadata, util};

#[derive(Deserialize, Debug)]
pub(super) struct ReproducibleBuild {
    image: String,
    image_digest: String,
    pub build_command: Option<String>,
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

        println!(
            "{}",
            util::indent_string(&format!(
                "{} {:#?}",
                "reproducible build metadata:".green(),
                build_meta
            ))
        );
        if build_meta.build_command.is_some() {
            println!(
                " {}", "using `build_command` in container from `[package.metadata.near.reproducible_build]` in Cargo.toml".cyan()
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
