use cargo_near::Cmd;
use clap::Parser;
use colored::Colorize;
use std::env;

fn main() {
    env_logger::init();

    match env::var("NO_COLOR") {
        Ok(v) if v != "0" => colored::control::set_override(false),
        _ => colored::control::set_override(atty::is(atty::Stream::Stderr)),
    }

    let Opts::Near(args) = Cmd::try_parse();
    match cargo_near::exec(args.cmd) {
        Ok(()) => {}
        Err(err) => {
            let err = format!("{:?}", err);
            let mut err_lines = err.lines();
            eprintln!(" {} {}", "✗".bright_red().bold(), err_lines.next().unwrap());
            for line in err_lines {
                eprintln!("   {}", line);
            }
            std::process::exit(1);
        }
    }
}
