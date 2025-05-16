use colored::Colorize;

use super::metadata;

impl super::Opts {
    pub fn get_cli_build_command_in_docker(
        &self,
        applied_build_meta: &metadata::AppliedReproducibleBuild,
    ) -> eyre::Result<Vec<String>> {
        let Some(manifest_command) = applied_build_meta.container_build_command.as_ref() else {
            return Err(eyre::eyre!(
                "`container_build_command` is expected to be non-empty (after validation)"
            ));
        };

        let section_name = metadata::section_name(self.variant.as_ref());
        println!(
            "{}`{}`{}",
            "using `container_build_command` from ".cyan(),
            section_name.magenta(),
            " in Cargo.toml".cyan()
        );
        self.append_env_suffix(
            manifest_command.clone(),
            applied_build_meta.passed_env.clone(),
        )
    }

    fn append_env_suffix(
        &self,
        mut manifest_command: Vec<String>,
        passed_env: Option<Vec<String>>,
    ) -> eyre::Result<Vec<String>> {
        if let Some(passed_env) = passed_env {
            let suffix_env = passed_env
                .into_iter()
                .filter(|env_key| std::env::var(env_key).is_ok())
                .flat_map(|env_key| {
                    println!(
                        "{}{}{}",
                        "detected environment build parameter, which has been set: `".cyan(),
                        env_key.yellow(),
                        "`".cyan(),
                    );
                    let value = std::env::var(&env_key).unwrap();
                    let pair = [env_key, value].join("=");
                    ["--env".to_string(), pair]
                })
                .collect::<Vec<_>>();

            if !suffix_env.is_empty() {
                let section_name = metadata::section_name(self.variant.as_ref());
                println!(
                    "{}{}{}",
                    "(listed in `".cyan(),
                    "passed_env".yellow(),
                    format!("` from `{}` in Cargo.toml)", section_name).cyan(),
                );
                println!();
            }

            manifest_command.extend(suffix_env);
        }

        Ok(manifest_command)
    }
}
