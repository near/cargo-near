use colored::Colorize;

use super::metadata;

impl super::Opts {
    pub fn get_cli_build_command_in_docker(
        &self,
        docker_build_meta: &metadata::ReproducibleBuild,
    ) -> eyre::Result<Vec<String>> {
        let Some(manifest_command) = docker_build_meta.container_build_command.as_ref() else {
            return Err(eyre::eyre!(
                "`container_build_command` is expected to be non-empty (after validation)"
            ));
        };
        println!(
            "{}`{}`{}",
            "using `container_build_command` from ".cyan(),
            "[package.metadata.near.reproducible_build]".magenta(),
            " in Cargo.toml".cyan()
        );
        self.append_env_suffix(
            manifest_command.clone(),
            docker_build_meta.passed_env.clone(),
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
                println!(
                    "{}{}{}",
                    "(listed in `".cyan(),
                    "passed_env".yellow(),
                    "` from `[package.metadata.near.reproducible_build]` in Cargo.toml)".cyan(),
                );
                println!();
            }

            manifest_command.extend(suffix_env);
        }

        Ok(manifest_command)
    }
}
