use cargo_near::Opts;
use clap::Parser;
use colored::Colorize;

fn main() {
    env_logger::init();

    let Opts::Near(args) = Opts::parse();
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
