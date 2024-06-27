use colored::Colorize;

pub(crate) fn handle_step<F, T>(msg: &str, f: F) -> color_eyre::eyre::Result<T>
where
    F: FnOnce() -> color_eyre::eyre::Result<T>,
{
    eprintln!("{} {}", "•".bold().cyan(), msg);
    let result = f();
    if result.is_ok() {
        eprintln!("{} {}\n", "•".bold().cyan(), "done".bold().green());
    } else {
        eprintln!("{} {}\n", "•".bold().cyan(), "failed".bold().red());
    }
    result
}

pub(crate) fn print_step(msg: &str) {
    eprintln!("{} {}", "•".bold().cyan(), msg);
}

pub(crate) fn print_success(msg: &str) {
    eprintln!("{} {}", "✓".bold().green(), msg);
}
