#![allow(clippy::large_enum_variant)]

pub use near_cli_rs::CliResult;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod build_extended;
pub mod commands;
pub mod common;
pub mod types;
pub mod util;

pub use build_extended::{build as build_extended, OptsExtended as BuildOptsExtended};
pub use commands::build_command::build::run as build;
pub use commands::build_command::build::Opts as BuildOpts;
pub use util::CompilationArtifact as BuildArtifact;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Cmd {
    #[interactive_clap(subcommand)]
    pub opts: Opts,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// Near
pub enum Opts {
    #[strum_discriminants(strum(message = "near"))]
    /// Which cargo extension do you want to use?
    Near(NearArgs),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct NearArgs {
    #[interactive_clap(subcommand)]
    pub cmd: self::commands::NearCommand,
}
