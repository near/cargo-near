use interactive_clap::ToCliArgs;
pub use near_cli_rs::CliResult;
use std::env;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

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
    #[strum_discriminants(strum(message = "near   -   Near"))]
    /// Near
    Near(cargo_near::NearArgs),
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
                        std::env::args().next().as_deref().unwrap_or("./bos"),
                        shell_words::join(cli_cmd.to_cli_args())
                    );
                }
                return Err(err);
            }
        }
    }
}
