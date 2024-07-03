#[cfg(target_os = "linux")]
#[test]
fn test_docker_build() -> cargo_near::CliResult {
    let mut builder = env_logger::Builder::from_env(env_logger::Env::default());
    let _e = builder.try_init();
    log::warn!("path {:?}", std::env::args().collect::<Vec<_>>());
    let manifest_dir: camino::Utf8PathBuf = env!("CARGO_MANIFEST_DIR").into();

    let cargo_near::CliOpts::Near(cli_args) = cargo_near::Opts::try_parse_from("cargo near build")?;

    let cargo_path_parent = manifest_dir.join("docker-build-template");
    let cargo_path = cargo_path_parent.join("Cargo.toml");
    match cli_args.cmd {
        Some(cargo_near::commands::CliNearCommand::Abi(_cmd)) => {
            unreachable!("another cmd is set by `cargo near build`");
        }
        Some(cargo_near::commands::CliNearCommand::Build(cmd)) => {
            let args = {
                let mut args = cargo_near::commands::build_command::BuildCommand::from(cmd);
                args.manifest_path = Some(cargo_path.into());
                args
            };
            args.run(cargo_near::commands::build_command::BuildContext::Build)?;
        }
        _ => {
            unreachable!("another cmd is set by `cargo near build`");
        }
    }

    Ok(())
}
