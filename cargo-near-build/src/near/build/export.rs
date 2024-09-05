use crate::{env_keys, types::near::build::input::Opts};

pub fn nep_330_build_command(args: &Opts) -> eyre::Result<()> {
    tracing::debug!(
        "compute `CARGO_NEAR_BUILD_COMMAND`,  current executable: {:?}",
        std::env::args().collect::<Vec<_>>()
    );
    let env_value: Vec<String> = match std::env::args().next() {
        // this is for cli context, being called from `cargo-near` bin
        Some(cli_arg_0)
            if cli_arg_0.ends_with("cargo-near") || cli_arg_0.ends_with("cargo-near.exe") =>
        {
            let mut cmd: Vec<String> = vec!["cargo".into()];
            // skipping `cargo-near`
            cmd.extend(std::env::args().skip(1));
            cmd
        }
        // this is for lib context, when build method is called from code
        // where `cargo-near` is an unlikely name to be chosen for executable
        _ => {
            // NOTE: order of output of cli flags shouldn't be too important, as the version of
            // `cargo-near` used as lib will be fixed in `Cargo.lock`
            args.get_cli_build_command()
        }
    };

    std::env::set_var(
        env_keys::nep330::BUILD_COMMAND,
        serde_json::to_string(&env_value)?,
    );
    Ok(())
}
