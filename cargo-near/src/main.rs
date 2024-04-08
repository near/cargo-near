use interactive_clap::ToCliArgs;
pub use near_cli_rs::CliResult;
use std::env;
use std::io::Write;

use cargo_near::Cmd;

fn main() -> CliResult {
    let mut builder = env_logger::Builder::from_env(env_logger::Env::default());
    builder
        .format(|buf, record| {
            let ts = buf.timestamp_seconds();
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                ts,
                record.level(),
                record.args()
            )
        })
        .init();

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

    let console_command_path = if env::var("CARGO_HOME").is_ok() {
        "cargo".to_string()
    } else if let Ok(value) = env::var("CARGO") {
        value.clone()
    } else {
        env::args().next().unwrap_or("./cargo".to_string())
    };

    loop {
        match <Cmd as interactive_clap::FromCli>::from_cli(
            Some(cli.clone()),
            global_context.clone(),
        ) {
            interactive_clap::ResultFromCli::Ok(cli_cmd)
            | interactive_clap::ResultFromCli::Cancel(Some(cli_cmd)) => {
                eprintln!(
                    "Here is the console command if you ever need to re-run it again:\n{console_command_path} {}",
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
                        "Here is the console command if you ever need to re-run it again:\n{console_command_path} {}\n",
                        shell_words::join(cli_cmd.to_cli_args())
                    );
                }
                return Err(err);
            }
        }
    }
}
