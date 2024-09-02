use colored::Colorize;

pub fn handle_step<F, T>(msg: &str, f: F) -> eyre::Result<T>
where
    F: FnOnce() -> eyre::Result<T>,
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

pub fn step(msg: &str) {
    eprintln!("{} {}", "•".bold().cyan(), msg);
}

pub fn success(msg: &str) {
    eprintln!("{} {}", "✓".bold().green(), msg);
}
