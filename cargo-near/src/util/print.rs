use colored::Colorize;

pub(crate) fn handle_step<F, T>(msg: &str, f: F) -> color_eyre::eyre::Result<T>
where
    F: FnOnce() -> color_eyre::eyre::Result<T>,
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
