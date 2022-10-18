use cargo_near::Opts;
use clap::Parser;
use colored::Colorize;

fn main() {
    env_logger::init();

    let Opts::Near(args) = Opts::parse();
    match cargo_near::exec(args.cmd) {
        Ok(()) => {}
        Err(err) => {
            eprintln!(" {} {:?}", "✗".bright_red().bold(), err);
            std::process::exit(1);
        }
    }
}
