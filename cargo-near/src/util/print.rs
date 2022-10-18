use colored::Colorize;

pub(crate) fn handle_step<F, T>(msg: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    eprint!(" {} {}", "•".bold().cyan(), msg);
    let result = f();
    eprintln!("{}", "done".bold().green());
    result
}

pub(crate) fn print_step(msg: &str) {
    eprintln!(" {} {}", "•".bold().cyan(), msg);
}

pub(crate) fn print_success(msg: &str) {
    eprintln!(" {} {}", "✓".bold().green(), msg);
}
