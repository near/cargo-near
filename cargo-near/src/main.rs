use clap::Parser;
use colored::Colorize;

use cargo_near::Opts;

fn main() {
    env_logger::init();

    let Opts::Near(args) = Opts::parse();
    match cargo_near::exec(args.cmd) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{} {}", "error:".bright_red().bold(), format!("{:?}", err));
            std::process::exit(1);
        }
    }
}
