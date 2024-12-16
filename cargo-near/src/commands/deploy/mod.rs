mod actions {
    mod non_reproducible_wasm;
    mod reproducible_wasm;

    use strum::{EnumDiscriminants, EnumIter, EnumMessage};

    #[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
    #[strum_discriminants(derive(EnumMessage, EnumIter))]
    #[interactive_clap(context = near_cli_rs::GlobalContext)]
    pub enum Actions {
        #[strum_discriminants(strum(
            message = "build-non-reproducible-wasm  - Fast and simple build (recommended for use during local development)"
        ))]
        /// Fast and simple build (recommended for use during local development)
        BuildNonReproducibleWasm(self::non_reproducible_wasm::DeployOpts),
        #[strum_discriminants(strum(
            message = "build-reproducible-wasm      - Build requires [reproducible_build] section in Cargo.toml, and all changes committed and pushed to git (recommended for the production release)"
        ))]
        /// Build requires [reproducible_build] section in Cargo.toml, and all changes committed and pushed to git (recommended for the production release)
        BuildReproducibleWasm(self::reproducible_wasm::DeployOpts),
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Command {
    #[interactive_clap(subcommand)]
    actions: actions::Actions,
}
