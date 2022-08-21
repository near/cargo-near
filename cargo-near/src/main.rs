use clap::Parser;
use colored::Colorize;

use cargo_near::Opts;

mod rustc_wrapper;

fn main() {
    env_logger::init();

    match Opts::parse() {
        Opts::Near(args) => match cargo_near::exec(args.cmd) {
            Ok(()) => {}
            Err(err) => {
                eprintln!(
                    "{} {}",
                    "ERROR:".bright_red().bold(),
                    format!("{:?}", err).bright_red()
                );
                std::process::exit(1);
            }
        },
        Opts::Rustc(args) => rustc_wrapper::run(args),
    };
}
