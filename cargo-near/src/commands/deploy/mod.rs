mod actions {
    mod non_reproducible_wasm;

    
    use strum::{EnumDiscriminants, EnumIter, EnumMessage};

    #[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
    #[strum_discriminants(derive(EnumMessage, EnumIter))]
    #[interactive_clap(context = near_cli_rs::GlobalContext)]
    pub enum Actions {
        #[strum_discriminants(strum(
            message = "build-non-reproducible-wasm - Build runs on current filesystem state without many restrictions"
        ))]
        /// Build runs on current filesystem state without many restrictions
        BuildNonReproducibleWasm(self::non_reproducible_wasm::DeployOpts),
        #[strum_discriminants(strum(
            message = "build-reproducible-wasm - Build requires `docker` config added to Cargo.toml and `git`-committed state, which is NOT dirty"
        ))]
        /// Build requires `docker` config added to Cargo.toml and `git`-committed state, which is NOT dirty
        BuildReproducibleWasm,
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Command {
    #[interactive_clap(subcommand)]
    actions: actions::Actions,
}
