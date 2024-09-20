use std::env;
use std::io::IsTerminal;

use cargo_near_build::env_keys;
use colored::Colorize;
use interactive_clap::ToCliArgs;

use indicatif::ProgressStyle;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::fmt::format;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{filter::filter_fn, prelude::*};

pub use near_cli_rs::CliResult;

use cargo_near::Cmd;

fn main() -> CliResult {
    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    if env::var("RUST_LOG").is_ok() {
        let environment = if std::env::var(env_keys::nep330::BUILD_ENVIRONMENT).is_ok() {
            "container".cyan()
        } else {
            "host".purple()
        };
        let my_formatter =
            cargo_near::types::my_formatter::MyFormatter::from_environment(environment);

        let format = format::debug_fn(move |writer, _field, value| write!(writer, "{:?}", value));

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(my_formatter)
                    .fmt_fields(format)
                    .with_filter(EnvFilter::from_default_env())
                    .with_filter(filter_fn(|metadata| metadata.target() != "near_teach_me")),
            )
            .init();
    } else if cli.teach_me {
        let env_filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::WARN.into())
            .add_directive("near_teach_me=info".parse()?)
            .add_directive("near_cli_rs=info".parse()?)
            .add_directive("cargo_near=info".parse()?);
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_target(false),
            )
            .with(env_filter)
            .init();
    } else {
        let indicatif_layer = IndicatifLayer::new()
            .with_progress_style(
                ProgressStyle::with_template(
                    "{spinner:.blue}{span_child_prefix} {span_name} {msg} {span_fields}",
                )
                .unwrap()
                .tick_strings(&[
                    "▹▹▹▹▹",
                    "▸▹▹▹▹",
                    "▹▸▹▹▹",
                    "▹▹▸▹▹",
                    "▹▹▹▸▹",
                    "▹▹▹▹▸",
                    "▪▪▪▪▪",
                ]),
            )
            .with_span_child_prefix_symbol("↳ ");
        let env_filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::WARN.into())
            .add_directive("near_cli_rs=info".parse()?)
            .add_directive("cargo_near=info".parse()?);
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_writer(indicatif_layer.get_stderr_writer()),
            )
            .with(indicatif_layer)
            .with(env_filter)
            .init();
    };

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
