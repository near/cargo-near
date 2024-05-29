use colored::Colorize;

const ERR_SANITY: &str = "`docker` sanity check failed!";

const PERM_DENIED_STATUS: i32 = 126;

pub fn check() -> color_eyre::eyre::Result<()> {
    let mut docker_cmd = std::process::Command::new("docker");
    docker_cmd.args(["run", "hello-world"]);
    let output_result = docker_cmd.output();
    let output = super::handle_command_io_error(
        &docker_cmd,
        output_result,
        color_eyre::eyre::eyre!(ERR_SANITY),
    )?;

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr)?;
        println!();
        println!("{}", stderr.yellow());
        if permission_denied(&output.status, stderr)? {
            println!("{}", "Permission denied!".cyan());
            super::print_installation_links(true);
        } else {
            super::print_installation_links(false);
        }
        super::print_command_status(output.status, docker_cmd);
        return Err(color_eyre::eyre::eyre!(ERR_SANITY));
    }
    Ok(())
}

fn permission_denied(
    status: &std::process::ExitStatus,
    stderr: &str,
) -> color_eyre::eyre::Result<bool> {
    let exit_code_match = status.code().unwrap_or(-1) == PERM_DENIED_STATUS;
    let stderr_match = stderr.to_lowercase().contains("permission denied");
    Ok(exit_code_match || stderr_match)
}
