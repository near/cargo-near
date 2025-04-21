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
    pub mod non_reproducible_wasm;
    pub mod reproducible_wasm;

    use strum::{EnumDiscriminants, EnumIter, EnumMessage};

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
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Command {
    #[interactive_clap(subcommand)]
    actions: actions::Actions,
}
