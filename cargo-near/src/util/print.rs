use colored::Colorize;

pub(crate) fn handle_step<F, T>(msg: &str, f: F) -> color_eyre::eyre::Result<T>
where
    F: FnOnce() -> color_eyre::eyre::Result<T>,
{
    eprintln!(" {} {}", "•".bold().cyan(), msg);
    let result = f();
    if result.is_ok() {
        eprintln!(" {} {}\n", "•".bold().cyan(), "done".bold().green());
    } else {
        eprintln!(" {} {}\n", "•".bold().cyan(), "failed".bold().red());
    }
    result
}

pub(crate) fn print_step(msg: &str) {
    eprintln!(" {} {}", "•".bold().cyan(), msg);
}

pub(crate) fn print_success(msg: &str) {
    eprintln!(" {} {}", "✓".bold().green(), msg);
}

pub(crate) fn indent_string(msg: &str) -> String {
    let indent = " ";
    let newline_indent = format!("\n{}", indent);
    let lines = msg.split('\n').collect::<Vec<_>>();
    let mut res = String::new();
    res.push_str(indent);

    res.push_str(&lines.join(&newline_indent));
    res
}
