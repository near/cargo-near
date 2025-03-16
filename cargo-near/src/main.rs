use std::env;
use std::io::IsTerminal;

use colored::Colorize;
use interactive_clap::ToCliArgs;

pub use near_cli_rs::CliResult;

use cargo_near::{
    commands::build::actions::non_reproducible_wasm as build_non_reproducible_wasm, setup_tracing,
    CliOpts, Cmd, Opts,
};

/// this part of cli setup doesn't depend on command arguments in any way
fn pre_setup() -> CliResult {
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
    Ok(())
}

fn main() -> CliResult {
    pre_setup()?;

    build_non_reproducible_wasm::rule::enforce_this_program_args()?;

    let cli_cmd = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };
    let config = near_cli_rs::config::Config::get_config_toml()?;
    let global_context = near_cli_rs::GlobalContext {
        config,
        verbosity: near_cli_rs::Verbosity::Interactive,
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
