use interactive_clap::ToCliArgs;
pub use near_cli_rs::CliResult;
use std::env;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod abi_command;
mod build_command;
pub mod common;
pub mod types;
pub mod util;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
struct Cmd {
    #[interactive_clap(subcommand)]
    opts: Opts,
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
    pub cmd: NearCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum NearCommand {
    #[strum_discriminants(strum(
        message = "build   -   Build a NEAR contract and optionally embed ABI"
    ))]
    /// Build a NEAR contract and optionally embed ABI
    Build(self::build_command::BuildCommand),
    #[strum_discriminants(strum(message = "abi     -   Generates ABI for the contract"))]
    /// Generates ABI for the contract
    Abi(self::abi_command::AbiCommand),
}

fn main() -> CliResult {
    env_logger::init();

    match env::var("NO_COLOR") {
        Ok(v) if v != "0" => colored::control::set_override(false),
        _ => colored::control::set_override(atty::is(atty::Stream::Stderr)),
    }

    let config = near_cli_rs::common::get_config_toml()?;

    color_eyre::install()?;

    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    let global_context = near_cli_rs::GlobalContext {
        config,
        offline: false,
    };

    loop {
        match <Cmd as interactive_clap::FromCli>::from_cli(
            Some(cli.clone()),
            global_context.clone(),
        ) {
            interactive_clap::ResultFromCli::Ok(cli_cmd)
            | interactive_clap::ResultFromCli::Cancel(Some(cli_cmd)) => {
                eprintln!(
                    "Your console command:\n{} {}",
                    std::env::args().next().as_deref().unwrap_or("./cargo-near"),
                    shell_words::join(cli_cmd.to_cli_args())
                );
                return Ok(());
            }
            interactive_clap::ResultFromCli::Cancel(None) => {
                eprintln!("Goodbye!");
                return Ok(());
            }
            interactive_clap::ResultFromCli::Back => {}
            interactive_clap::ResultFromCli::Err(optional_cli_cmd, err) => {
                if let Some(cli_cmd) = optional_cli_cmd {
                    eprintln!(
                        "Your console command:\n{} {}",
                        std::env::args().next().as_deref().unwrap_or("./cargo-near"),
                        shell_words::join(cli_cmd.to_cli_args())
                    );
                }
                return Err(err);
            }
        }
    }
}
