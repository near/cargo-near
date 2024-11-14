use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod non_reproducible_wasm; 
mod reproducible_wasm;


#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = BuildCommandlContext)]
pub enum BuildCommandActions {
    #[strum_discriminants(strum(
        message = "non-reproducible-wasm - Runs on current filesystem state without many restrictions"
    ))]
    /// Runs on current filesystem state without many restrictions
    NonReproducibleWasm(self::non_reproducible_wasm::Opts),
    #[strum_discriminants(strum(
        message = "reproducible-wasm - Requires `docker` config added to Cargo.toml and `git`-committed state, which is NOT dirty"
    ))]
    /// Requires `docker` config added to Cargo.toml and `git`-committed state, which is NOT dirty
    ReproducibleWasm(self::reproducible_wasm::Opts),
    
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct  BuildCommand {
    #[interactive_clap(subcommand)]
    actions: BuildCommandActions,
}


impl BuildCommand {
    // fn validate_env_opt(&self) -> color_eyre::eyre::Result<()> {
    //     for pair in self.env.iter() {
    //         pair.split_once('=').ok_or(color_eyre::eyre::eyre!(
    //             "invalid \"key=value\" environment argument (must contain '='): {}",
    //             pair
    //         ))?;
    //     }
    //     Ok(())
    // }
}


// fn get_env_key_vals(input: Vec<String>) -> Vec<(String, String)> {
//     let iterator = input.iter().flat_map(|pair_string| {
//         pair_string
//             .split_once('=')
//             .map(|(env_key, value)| (env_key.to_string(), value.to_string()))
//     });

//     let dedup_map: HashMap<String, String> = HashMap::from_iter(iterator);

//     let result = dedup_map.into_iter().collect();
//     tracing::info!(
//         target: "near_teach_me",
//         parent: &tracing::Span::none(),
//         "Passed additional environment pairs:\n{}",
//         near_cli_rs::common::indent_payload(&format!("{:#?}", result))
//     );
//     result
// }


impl From<BuildCommandlContext> for cargo_near_build::BuildContext {
    fn from(_value: BuildCommandlContext) -> Self {
        Self::Build
    }
}


#[derive(Debug, Clone)]
pub struct BuildCommandlContext;

impl BuildCommandlContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        _scope: &<BuildCommandActions as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self)
    }
}
