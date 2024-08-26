use cargo_near_build::docker_build_types::metadata::ReproducibleBuild;
use colored::Colorize;

pub fn check(docker_build_meta: &ReproducibleBuild) -> color_eyre::eyre::Result<()> {
    let docker_image = docker_build_meta.concat_image();
    println!("{} {}", "docker image to be used:".green(), docker_image);
    println!();

    let mut docker_cmd = docker_pull_cmd(&docker_image);

    let err_report = format!("Image `{}` could not be found in registry!", docker_image);
    let status_result = docker_cmd.status();
    let status = super::handle_command_io_error(
        &docker_cmd,
        status_result,
        color_eyre::eyre::eyre!(err_report.clone()),
    )?;
    if !status.success() {
        super::print_command_status(status, docker_cmd);
        return Err(color_eyre::eyre::eyre!(err_report));
    }
    Ok(())
}

fn docker_pull_cmd(image: &str) -> std::process::Command {
    let docker_cmd: std::process::Command = {
        let docker_args = {
            let mut docker_args = vec!["pull"];
            docker_args.push(image);
            docker_args
        };

        let mut docker_cmd = std::process::Command::new("docker");
        docker_cmd.arg("image");
        docker_cmd.args(docker_args);
        docker_cmd
    };
    docker_cmd
}
