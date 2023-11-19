use color_eyre::eyre::{ContextCompat, WrapErr};

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
        let project_dir: &std::path::Path = scope.project_dir.as_ref();
        let project_name = project_dir
            .file_name()
            .wrap_err("Could not extract project name from project path")?
            .to_str()
            .wrap_err("Project name has to be a valid UTF-8 string")?;

        for new_project_file in NEW_PROJECT_FILES {
            let new_file_path = project_dir.join(new_project_file.file_path);
            std::fs::create_dir_all(&new_file_path.parent().wrap_err_with(|| {
                format!("Impossible to get parent for `{}`", new_file_path.display())
            })?)?;
            std::fs::write(
                &new_file_path,
                new_project_file
                    .content
                    .replace("cargo-near-new-project-name", project_name),
            )
            .wrap_err_with(|| format!("Failed to write to file: {}", new_file_path.display()))?;
        }

        std::process::Command::new("git")
            .arg("init")
            .current_dir(&project_dir)
            .output()
            .wrap_err("Failed to execute process: `git init`")?;

        println!("New project is created at '{}'\n", project_dir.display());
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
        content: include_str!("new-project-template/.github/workflows/deploy-production.yml"),
    },
    NewProjectFile {
        file_path: ".github/workflows/deploy-staging.yml",
        content: include_str!("new-project-template/.github/workflows/deploy-staging.yml"),
    },
    NewProjectFile {
        file_path: ".github/workflows/test.yml",
        content: include_str!("new-project-template/.github/workflows/test.yml"),
    },
    NewProjectFile {
        file_path: ".github/workflows/undeploy-staging.yml",
        content: include_str!("new-project-template/.github/workflows/undeploy-staging.yml"),
    },
    NewProjectFile {
        file_path: "src/lib.rs",
        content: include_str!("new-project-template/src/lib.rs"),
    },
    NewProjectFile {
        file_path: "tests/test_basics.rs",
        content: include_str!("new-project-template/tests/test_basics.rs"),
    },
    NewProjectFile {
        file_path: ".gitignore",
        content: include_str!("new-project-template/.gitignore"),
    },
    NewProjectFile {
        file_path: "Cargo.toml",
        content: include_str!("new-project-template/Cargo.toml"),
    },
    NewProjectFile {
        file_path: "README.md",
        content: include_str!("new-project-template/README.md"),
    },
    NewProjectFile {
        file_path: "rust-toolchain.toml",
        content: include_str!("new-project-template/rust-toolchain.toml"),
    },
];
