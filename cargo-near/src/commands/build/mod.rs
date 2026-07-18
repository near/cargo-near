pub mod context {
    #[derive(Debug, Clone)]
    pub struct Context;

    impl From<Context> for cargo_near_build::docker::BuildContext {
        fn from(_value: Context) -> Self {
            Self::Build
        }
    }

    impl Context {
        pub fn from_previous_context(
            _previous_context: near_cli_rs::GlobalContext,
            _scope: &<super::actions::Actions as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
        ) -> color_eyre::eyre::Result<Self> {
            Ok(Self)
        }
    }
}

pub mod actions {
    pub mod list;
    pub mod non_reproducible_wasm;
    pub mod reproducible_wasm;

    use strum::{EnumDiscriminants, EnumIter, EnumMessage};

    /// Warn (on stderr) about workspace members that were skipped because they lack a
    /// `[package.metadata.near.reproducible_build]` section, so a skipped member (e.g. a
    /// contract that hasn't added the section yet) is visible rather than dropped silently.
    /// Caps the listed names for large workspaces. Shared by `build list` and `--workspace`.
    pub(crate) fn warn_skipped_members(workspace: &cargo_near_build::list::Workspace) {
        use colored::Colorize;
        if workspace.skipped.is_empty() {
            return;
        }
        const MAX_LISTED: usize = 10;
        let total = workspace.skipped.len();
        let mut names = workspace.skipped[..total.min(MAX_LISTED)].join(", ");
        if total > MAX_LISTED {
            names.push_str(&format!(" (+{} more)", total - MAX_LISTED));
        }
        eprintln!(
            "{}",
            format!(
                "Skipped {total} workspace member(s) without a \
                 [package.metadata.near.reproducible_build] section: {names}"
            )
            .yellow()
        );
    }

    /// Errors if two contracts in the workspace would write the same wasm filename to a shared
    /// `--out-dir`. Without `--out-dir`, each non-root workspace member writes to its own default
    /// subdirectory, so matching filenames do not necessarily collide.
    pub(crate) fn assert_unique_workspace_outputs(
        workspace: &cargo_near_build::list::Workspace,
        has_shared_out_dir: bool,
    ) -> color_eyre::eyre::Result<()> {
        if !has_shared_out_dir {
            return Ok(());
        }

        let mut seen = std::collections::HashMap::<String, String>::new();
        for contract in &workspace.contracts {
            let output = contract.wasm_filename();
            if let Some(previous) = seen.insert(output.clone(), contract.name.clone()) {
                return Err(color_eyre::eyre::eyre!(
                    "contracts `{previous}` and `{}` both build to `{output}`; \
                     rename one so their workspace outputs don't collide",
                    contract.name
                ));
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use cargo_near_build::camino::Utf8PathBuf;
        use cargo_near_build::list::{Workspace, WorkspaceContract};

        use super::assert_unique_workspace_outputs;

        fn workspace_with_normalized_name_collision() -> Workspace {
            Workspace {
                root: Utf8PathBuf::from("/workspace"),
                contracts: vec![
                    WorkspaceContract {
                        name: "foo-bar".to_string(),
                        manifest_path: Utf8PathBuf::from("Cargo.toml"),
                        variants: vec![None],
                    },
                    WorkspaceContract {
                        name: "foo_bar".to_string(),
                        manifest_path: Utf8PathBuf::from("member/Cargo.toml"),
                        variants: vec![None],
                    },
                ],
                skipped: vec![],
            }
        }

        #[test]
        fn matching_filenames_are_allowed_without_shared_out_dir() {
            let workspace = workspace_with_normalized_name_collision();

            assert!(assert_unique_workspace_outputs(&workspace, false).is_ok());
        }

        #[test]
        fn matching_filenames_are_rejected_with_shared_out_dir() {
            let workspace = workspace_with_normalized_name_collision();

            let error = assert_unique_workspace_outputs(&workspace, true).unwrap_err();
            let message = error.to_string();
            assert!(message.contains("foo-bar"));
            assert!(message.contains("foo_bar"));
            assert!(message.contains("foo_bar.wasm"));
        }
    }

    #[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
    #[strum_discriminants(derive(EnumMessage, EnumIter))]
    #[interactive_clap(input_context = near_cli_rs::GlobalContext)]
    #[interactive_clap(output_context = super::context::Context)]
    pub enum Actions {
        #[strum_discriminants(strum(
            message = "non-reproducible-wasm  - Fast and simple (recommended for use during local development)"
        ))]
        /// Fast and simple (recommended for use during local development)
        NonReproducibleWasm(self::non_reproducible_wasm::BuildOpts),
        #[strum_discriminants(strum(
            message = "reproducible-wasm      - Requires [reproducible_build] section in Cargo.toml, and all changes committed to git (recommended for the production release)"
        ))]
        /// Requires `[reproducible_build]` section in Cargo.toml, and all changes committed to git (recommended for the production release)
        ReproducibleWasm(self::reproducible_wasm::BuildOpts),
        #[strum_discriminants(strum(
            message = "list                   - List workspace contracts and their reproducible-build variants (supports --json for a CI build matrix)"
        ))]
        /// List workspace contracts and their reproducible-build variants (supports `--json` for a CI build matrix)
        List(self::list::ListOpts),
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Command {
    #[interactive_clap(subcommand)]
    actions: actions::Actions,
}
