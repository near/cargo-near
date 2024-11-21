use std::io::IsTerminal;
use std::{env, ops::Index};

use cargo_near_build::env_keys;
use colored::Colorize;
use interactive_clap::ToCliArgs;

pub use near_cli_rs::CliResult;

use cargo_near::{setup_tracing, CliOpts, Cmd, Opts};

fn main() -> CliResult {
    let args = std::env::args().collect::<Vec<_>>();
    if std::env::var(env_keys::nep330::BUILD_ENVIRONMENT).is_ok() {
        println!("enforcing stuff in docker");
        if args.len() < 4 {
            return Err(color_eyre::eyre::eyre!("it's not cool in docker"));
        }
        if args[1..4] != ["near", "build", "non-reproducible-wasm"] {
            return Err(color_eyre::eyre::eyre!("it's not cool in docker"));
        }
        // TODO: clean logic to enforce `binary near build non-reproducible` prefix of command
    }
    let cli_cmd = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };
    let config = near_cli_rs::config::Config::get_config_toml()?;
    let global_context = near_cli_rs::GlobalContext {
        config,
        teach_me: false,
        offline: false,
    };

    let cli_opts = match cli_cmd.clone().opts {
        Some(cli_opts) => cli_opts,
        None => match Opts::choose_variant(global_context.clone()) {
            interactive_clap::ResultFromCli::Ok(cli_opts) => cli_opts,
            interactive_clap::ResultFromCli::Err(_optional_cli_cmd, err) => return Err(err),
            _ => return Ok(()),
        },
    };
    let CliOpts::Near(cli_near_args) = cli_opts.clone();

    setup_tracing(env::var("RUST_LOG").is_ok(), cli_near_args.teach_me)?;

    match env::var("NO_COLOR") {
        Ok(v) if v != "0" => colored::control::set_override(false),
        _ => colored::control::set_override(std::io::stderr().is_terminal()),
    }

    #[cfg(not(debug_assertions))]
    let display_env_section = false;
    #[cfg(debug_assertions)]
    let display_env_section = true;
    color_eyre::config::HookBuilder::default()
        .display_env_section(display_env_section)
        .install()?;

    let console_command_path = if env::var("CARGO_HOME").is_ok() {
        "cargo".to_string()
    } else if let Ok(value) = env::var("CARGO") {
        value.clone()
    } else {
        env::args().next().unwrap_or("./cargo".to_string())
    };

    let console_command_path = console_command_path.yellow();

    loop {
        match <Opts as interactive_clap::FromCli>::from_cli(
            Some(cli_opts.clone()),
            global_context.clone(),
        ) {
            interactive_clap::ResultFromCli::Ok(cli_opts)
            | interactive_clap::ResultFromCli::Cancel(Some(cli_opts)) => {
                eprintln!(
                    "Here is the console command if you ever need to re-run it again:\n{console_command_path} {}",
                    shell_words::join(cli_opts.to_cli_args()).yellow()
                );
                return Ok(());
            }
            interactive_clap::ResultFromCli::Cancel(None) => {
                eprintln!("Goodbye!");
                return Ok(());
            }
            interactive_clap::ResultFromCli::Back => {}
            interactive_clap::ResultFromCli::Err(optional_cli_opts, err) => {
                if let Some(cli_opts) = optional_cli_opts {
                    eprintln!(
                        "Here is the console command if you ever need to re-run it again:\n{console_command_path} {}\n",
                        shell_words::join(cli_opts.to_cli_args()).yellow()
                    );
                }
                return Err(err);
            }
        }
    }
}
