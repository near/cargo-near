use colored::Colorize;

pub(crate) fn handle_step<F, T, E>(msg: &str, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    eprint!(" {} {}", "•".bold().cyan(), msg);
    let result = f();
    if result.is_ok() {
        eprintln!("{}", "done".bold().green());
    } else {
        eprintln!("{}", "failed".bold().red());
    }
    result
}

pub(crate) fn print_step(msg: &str) {
    eprintln!(" {} {}", "•".bold().cyan(), msg);
}

pub(crate) fn print_success(msg: &str) {
    eprintln!(" {} {}", "✓".bold().green(), msg);
}

pub(crate) fn print_error(msg: &str) {
    eprintln!(" {} {}", "✗".bold().red(), msg);
}
