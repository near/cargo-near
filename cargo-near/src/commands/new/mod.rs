use color_eyre::eyre::{ContextCompat, WrapErr};
use std::io::Write;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = NewContext)]
pub struct New {
    /// Enter a new project name (path to the project) to create a contract:
    pub project_dir: near_cli_rs::types::path_buf::PathBuf,
}

#[derive(Debug, Clone)]
pub struct NewContext;

impl NewContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        scope: &<New as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let project_dir = scope.project_dir.to_string();
        let project_name = project_dir.rsplit('/').next().wrap_err("Internal error!")?;

        for new_project_file in NEW_PROJECT_FILES {
            let mut folder_path = std::path::PathBuf::from(&project_dir);
            let file_path = std::path::PathBuf::from(new_project_file.file_path);
            folder_path.push(file_path.parent().wrap_err_with(|| {
                format!("Impossible to get parent for `{}`", file_path.display())
            })?);
            std::fs::create_dir_all(&folder_path)?;
            let path = folder_path.join(file_path.file_name().wrap_err_with(|| {
                format!("Impossible to get filename for `{}`", file_path.display())
            })?);
            std::fs::File::create(&path)
                .wrap_err_with(|| format!("Failed to create file: {}", path.display()))?
                .write(
                    new_project_file
                        .content
                        .replace("cargo-near-new-project", project_name)
                        .as_bytes(),
                )
                .wrap_err_with(|| format!("Failed to write to file: {}", path.display()))?;
        }

        std::process::Command::new("git")
            .arg("init")
            .current_dir(&project_dir)
            .output()
            .wrap_err("Failed to execute process: `git init`")?;

        println!("New project is created at '{project_dir}'\n");
        println!("Now you can build, deploy, and finish CI setup for automatic deployment:");
        println!("1. `cargo near build`");
        println!("2. `cargo test`");
        println!("3. `cargo near deploy`");
        println!("4. Configure `NEAR_CONTRACT_STAGING_*` and `NEAR_CONTRACT_PRODUCTION_*` variables and secrets on GitHub to enable automatic deployment to staging and production. See more details in `.github/workflow/*` files.");

        Ok(Self)
    }
}

struct NewProjectFile {
    file_path: &'static str,
    content: &'static str,
}

const NEW_PROJECT_FILES: &[NewProjectFile] = &[
    NewProjectFile {
        file_path: ".github/workflows/deploy-production.yml",
        content: include_str!("prototype_for_project/.github/workflows/deploy-production.yml"),
    },
    NewProjectFile {
        file_path: ".github/workflows/deploy-staging.yml",
        content: include_str!("prototype_for_project/.github/workflows/deploy-staging.yml"),
    },
    NewProjectFile {
        file_path: ".github/workflows/test.yml",
        content: include_str!("prototype_for_project/.github/workflows/test.yml"),
    },
    NewProjectFile {
        file_path: ".github/workflows/undeploy-staging.yml",
        content: include_str!("prototype_for_project/.github/workflows/undeploy-staging.yml"),
    },
    NewProjectFile {
        file_path: "src/lib.rs",
        content: include_str!("prototype_for_project/src/lib.rs"),
    },
    NewProjectFile {
        file_path: "tests/test_basics.rs",
        content: include_str!("prototype_for_project/tests/test_basics.rs"),
    },
    NewProjectFile {
        file_path: ".gitignore",
        content: include_str!("prototype_for_project/.gitignore"),
    },
    NewProjectFile {
        file_path: "Cargo.toml",
        content: include_str!("prototype_for_project/Cargo.toml"),
    },
    NewProjectFile {
        file_path: "README.md",
        content: include_str!("prototype_for_project/README.md"),
    },
    NewProjectFile {
        file_path: "rust-toolchain.toml",
        content: include_str!("prototype_for_project/rust-toolchain.toml"),
    },
];
