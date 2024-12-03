pub mod context {
    #[derive(Debug, Clone)]
    pub struct Context;

    impl From<Context> for cargo_near_build::BuildContext {
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
    pub mod non_reproducible_wasm;
    pub mod reproducible_wasm;

    use strum::{EnumDiscriminants, EnumIter, EnumMessage};

    #[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
    #[strum_discriminants(derive(EnumMessage, EnumIter))]
    #[interactive_clap(input_context = near_cli_rs::GlobalContext)]
    #[interactive_clap(output_context = super::context::Context)]
    pub enum Actions {
        #[strum_discriminants(strum(
            message = "non-reproducible-wasm - Runs on current filesystem state without many restrictions"
        ))]
        /// Runs on current filesystem state without many restrictions
        NonReproducibleWasm(self::non_reproducible_wasm::BuildOpts),
        #[strum_discriminants(strum(
            message = "reproducible-wasm - Requires `docker` config added and (git)committed to Cargo.toml, runs on clean (git)working tree state"
        ))]
        /// Requires `docker` config added and (git)committed to Cargo.toml, runs on clean (git)working tree state
        ReproducibleWasm(self::reproducible_wasm::BuildOpts),
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Command {
    #[interactive_clap(subcommand)]
    actions: actions::Actions,
}
