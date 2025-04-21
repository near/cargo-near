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

#[cfg(feature = "build_internal")]
pub fn duration(start: Instant, activity: &str) {
    let duration = std::time::Duration::from_secs(start.elapsed().as_secs());
    println!(
        "    {} {} in {}",
        "Finished".bold().cyan(),
        activity,
        humantime::format_duration(duration)
    );
}

#[cfg(feature = "build_internal")]
pub fn duration_millis(start: Instant, activity: &str) {
    let duration = std::time::Duration::from_millis(start.elapsed().as_millis() as u64);
    println!(
        "    {} {} in {}",
        "Finished".bold().truecolor(90, 90, 90),
        activity,
        humantime::format_duration(duration)
    );
}

pub fn indent_payload(s: &str) -> String {
    use std::fmt::Write;

    let mut indented_string = String::new();
    indenter::indented(&mut indented_string)
        .with_str(" |    ")
        .write_str(s)
        .ok();
    indented_string
}
