use colored::Colorize;

use serde_json::Value;
use std::{collections::BTreeMap, thread, time::Duration};

pub(crate) mod parse;
mod validate;

pub struct AppliedReproducibleBuild {
    pub image: String,
    pub image_digest: String,
    pub passed_env: Option<Vec<String>>,
    pub container_build_command: Option<Vec<String>>,

    /// a cloneable git remote url,
    /// currently, only ones, starting with `https://`, are supported;
    /// parsed from `package.repository`
    pub repository: Option<url::Url>,

    /// indicator for used variant of reproducible build;
    /// present if the variant of build was used
    pub selected_variant: Option<String>,

    unknown_keys: BTreeMap<String, Value>,
}

pub(crate) fn section_name(variant: Option<&String>) -> String {
    let variant_suffix = variant
        .map(|name| format!(".variant.{name}"))
        .unwrap_or_default();

    format!("[package.metadata.near.reproducible_build{variant_suffix}]")
}

#[allow(clippy::write_literal)]
impl std::fmt::Display for AppliedReproducibleBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;

        if let Some(ref variant_name) = self.selected_variant {
            writeln!(f, "    {}: {}", "build variant", variant_name.yellow())?;
        } else {
            writeln!(f, "    {}: {}", "build variant", "<DEFAULT>")?;
        }

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
            "cloneable remote of git repository",
            self.repository
                .as_ref()
                .map(|url| url.as_ref())
                .unwrap_or("<empty>")
        )?;
        Ok(())
    }
}

impl AppliedReproducibleBuild {
    pub fn new(reproducible_build_parsed: &parse::ReproducibleBuild) -> Self {
        Self {
            image: reproducible_build_parsed.image.clone(),
            image_digest: reproducible_build_parsed.image_digest.clone(),
            passed_env: reproducible_build_parsed.passed_env.clone(),
            container_build_command: reproducible_build_parsed.container_build_command.clone(),
            repository: reproducible_build_parsed.repository.clone(),
            selected_variant: None,
            unknown_keys: reproducible_build_parsed.unknown_keys.clone(),
        }
    }

    fn inject_variant_build(
        &mut self,
        variant_name: &str,
        variant_build: &parse::VariantReproducibleBuild,
    ) {
        println!();
        println!(
            "{}{}{}",
            "Injecting variant build `.variant.".yellow(),
            variant_name.yellow(),
            "`:".yellow()
        );

        self.selected_variant = Some(variant_name.to_string());

        if let Some(new_image) = &variant_build.image {
            println!("    {}", "Changing image:".yellow());
            println!("        {} `{}`", "default:".red(), self.image);
            println!("        {} `{}`", "override:".green(), new_image);
            println!();

            self.image.clone_from(new_image);
        }

        if let Some(new_image_digest) = &variant_build.image_digest {
            println!("    {}", "Changing image_digest:".yellow());
            println!("        {} `{}`", "default:".red(), self.image_digest);
            println!("        {} `{}`", "override:".green(), new_image_digest);
            println!();

            self.image_digest.clone_from(new_image_digest);
        }

        if let Some(new_passed_env) = &variant_build.passed_env {
            println!("    {}", "Changing passed_env:".yellow());

            if let Some(original_passed_env) = &self.passed_env {
                println!("        {} `{:?}`", "default:".red(), original_passed_env);
            } else {
                println!("        {} `{}`", "default:".red(), "<ABSENT>".green());
            }

            println!("        {} `{:?}`", "override:".green(), new_passed_env);
            println!();

            self.passed_env = Some(new_passed_env.clone());
        }

        if let Some(new_container_command) = &variant_build.container_build_command {
            println!("    {}", "Changing container_build_command:".yellow());

            if let Some(original_build_command) = &self.container_build_command {
                println!(
                    "        {} `{:?}`",
                    "default:".red(),
                    original_build_command
                );
            } else {
                println!("        {} `{}`", "default:".red(), "<ABSENT>".yellow());
            }

            println!(
                "        {} `{:?}`",
                "override:".green(),
                new_container_command
            );
            println!();

            self.container_build_command = Some(new_container_command.clone());
        }

        self.unknown_keys.extend(variant_build.unknown_keys.clone());
    }

    pub fn concat_image(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.image);
        result.push('@');
        result.push_str(&self.image_digest);
        result
    }
}

impl parse::ReproducibleBuild {
    /// Apply the variant of `[package.metadata.near.reproducible_build]` using the `variant_name`;
    /// if `variant_name` is empty - return default `[package.metadata.near.reproducible_build]`
    pub fn apply_variant_or_default(
        self,
        variant_name: Option<&str>,
    ) -> eyre::Result<AppliedReproducibleBuild> {
        let mut applied_variant = AppliedReproducibleBuild::new(&self);
        if let Some(name) = variant_name {
            if let Some(variant) = self.variants_map.get(name) {
                applied_variant.inject_variant_build(name, variant);
            } else {
                println!(
                    "{}{}{}",
                    "Build variant called `".yellow(),
                    name.yellow(),
                    "` was not found in Cargo.toml...".yellow()
                );
                thread::sleep(Duration::new(7, 0));
                println!();
                println!(
                    "{}{}{}{}{}",
                    "You can add and commit ".cyan(),
                    "`[package.metadata.near.reproducible_build.variant.".magenta(),
                    name.magenta(),
                    "]` ".magenta(),
                    "to your contract's Cargo.toml:".cyan()
                );

                thread::sleep(Duration::new(12, 0));

                return Err(eyre::eyre!(
                    "Missing `[package.metadata.near.reproducible_build.variant.{}]` in Cargo.toml",
                    name
                ));
            }
        }
        Ok(applied_variant)
    }
}
