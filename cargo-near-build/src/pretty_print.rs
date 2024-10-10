use std::time::Instant;

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

pub fn duration(start: Instant, activity: &str) {
    let duration = std::time::Duration::from_secs(start.elapsed().as_secs());
    println!(
        "    {} {} in {:#}",
        "Finished".bold().cyan(),
        activity,
        duration_human::DurationHuman::from(duration)
    );
}
