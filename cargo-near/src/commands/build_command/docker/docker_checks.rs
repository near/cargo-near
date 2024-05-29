use colored::Colorize;

pub use self::hello_world::check as sanity_check;
pub use self::pull_image::check as pull_image;

mod hello_world;
mod pull_image;

pub(super) fn handle_command_io_error<T>(
    command: &std::process::Command,
    command_result: std::io::Result<T>,
    report: color_eyre::eyre::Report,
) -> color_eyre::eyre::Result<T> {
    match command_result {
        Ok(result) => Ok(result),
        Err(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
            println!();
            println!("{}", "`docker` executable isn't available".yellow());
            print_installation_links(false);
            print_non_docker_suggestion();
            Err(report)
        }
        Err(io_err) => {
            println!();
            println!(
                "{}",
                format!(
                    "Error obtaining status from executing command `{:?}`",
                    command
                )
                .yellow()
            );
            println!("{}", format!("Error `{:?}`", io_err).yellow());
            print_non_docker_suggestion();
            Err(report)
        }
    }
}

fn print_installation_links(permission_denied: bool) {
    match std::env::consts::OS {
        "linux" => {
            println!(
                "{} {}",
                "Please, follow instructions to correctly install Docker Engine on".cyan(),
                "https://docs.docker.com/engine/install/".magenta()
            );
            if permission_denied {
                println!(
                    "{} {} {} `{}` {}",
                    "Please, pay special attention to".cyan(),
                    "https://docs.docker.com/engine/install/linux-postinstall/".magenta(),
                    "section regarding your".cyan(),
                    "permission denied".magenta(),
                    "problem".cyan(),
                );
            }
        }

        "macos" => {
            println!(
                "{} {}",
                "Please, follow instructions to correctly install Docker Desktop on".cyan(),
                "https://docs.docker.com/desktop/install/mac-install/".magenta()
            );
        }
        "windows" => {
            println!(
                "{} {}",
                "Please, follow instructions to correctly install Docker Desktop on".cyan(),
                "https://docs.docker.com/desktop/install/windows-install/".magenta()
            );
        }
        _ => {
            println!("{} {}", 
                "Please, make sure to follow instructions to correctly install Docker Engine/Desktop on".cyan(),
                "https://docs.docker.com/engine/install/".magenta()
            );
        }
    }
}

pub(super) fn print_command_status(
    status: std::process::ExitStatus,
    command: std::process::Command,
) {
    println!();
    let command = {
        let mut args = vec![command.get_program().to_string_lossy().to_string()];
        args.extend(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().to_string()),
        );
        args.join(" ")
    };

    println!(
        "{}",
        format!(
            "See output above ↑↑↑.\nCommand `{}` failed with: {status}.",
            command
        )
        .yellow()
    );
    print_non_docker_suggestion();
}

fn print_non_docker_suggestion() {
    println!(
        "{}",
        "You can choose to opt out into non-docker build behaviour by using `--no-docker` flag."
            .cyan()
    );
}
