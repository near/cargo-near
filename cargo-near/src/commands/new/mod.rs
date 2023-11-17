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
        const SOURCE_DIR: &str = "./cargo-near/src/commands/new/prototype_for_project/";
        let new_project_dir = scope.project_dir.clone();

        std::process::Command::new("mkdir")
            .arg(&new_project_dir)
            .output()
            .expect("failed to execute process");

        std::process::Command::new("cp")
            .arg("-r")
            .arg(SOURCE_DIR)
            .arg(&new_project_dir)
            .output()
            .expect("failed to execute process");

        std::process::Command::new("git")
            .arg("init")
            .current_dir(&new_project_dir)
            .output()
            .expect("failed to execute process");

        Ok(Self)
    }
}
