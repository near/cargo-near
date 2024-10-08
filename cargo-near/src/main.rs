use std::env;
use std::io::IsTerminal;

use cargo_near_build::env_keys;
use colored::Colorize;
use interactive_clap::ToCliArgs;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt::format, prelude::*};

pub use near_cli_rs::CliResult;

use cargo_near::Cmd;

fn main() -> CliResult {
    let environment = if std::env::var(env_keys::nep330::BUILD_ENVIRONMENT).is_ok() {
        "container".cyan()
    } else {
        "host".purple()
    };
    let my_formatter = cargo_near::types::my_formatter::MyFormatter::from_environment(environment);

    let format = format::debug_fn(move |writer, _field, value| write!(writer, "{:?}", value));

    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(my_formatter)
                .fmt_fields(format)
                .with_filter(env_filter),
        )
        .init();

    match env::var("NO_COLOR") {
        Ok(v) if v != "0" => colored::control::set_override(false),
        _ => colored::control::set_override(std::io::stderr().is_terminal()),
    }

    let config = near_cli_rs::config::Config::get_config_toml()?;

    #[cfg(not(debug_assertions))]
    let display_env_section = false;
    #[cfg(debug_assertions)]
    let display_env_section = true;
    color_eyre::config::HookBuilder::default()
        .display_env_section(display_env_section)
        .install()?;
    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    let global_context = near_cli_rs::GlobalContext {
        config,
        teach_me: false,
        offline: false,
    };

    let console_command_path = if env::var("CARGO_HOME").is_ok() {
        "cargo".to_string()
    } else if let Ok(value) = env::var("CARGO") {
        value.clone()
    } else {
        env::args().next().unwrap_or("./cargo".to_string())
    };

    let console_command_path = console_command_path.yellow();

    loop {
        match <Cmd as interactive_clap::FromCli>::from_cli(
            Some(cli.clone()),
            global_context.clone(),
        ) {
            interactive_clap::ResultFromCli::Ok(cli_cmd)
            | interactive_clap::ResultFromCli::Cancel(Some(cli_cmd)) => {
                eprintln!(
                    "Here is the console command if you ever need to re-run it again:\n{console_command_path} {}",
                    shell_words::join(cli_cmd.to_cli_args()).yellow()
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
                        "Here is the console command if you ever need to re-run it again:\n{console_command_path} {}\n",
                        shell_words::join(cli_cmd.to_cli_args()).yellow()
                    );
                }
                return Err(err);
            }
        }
    }
}
