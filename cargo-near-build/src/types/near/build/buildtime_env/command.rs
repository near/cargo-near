use crate::env_keys;
use crate::types::near::build::input::Opts;

pub struct Nep330BuildCommand {
    value: String,
}

impl Nep330BuildCommand {
    pub fn compute(args: &Opts) -> eyre::Result<Self> {
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
                args.get_cli_command_for_lib_context()
            }
        };

        let command = Self::new(serde_json::to_string(&env_value)?);
        Ok(command)
    }
    fn new(value: String) -> Self {
        tracing::info!("{}={}", env_keys::nep330::BUILD_COMMAND, value);
        Self { value }
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push((env_keys::nep330::BUILD_COMMAND, self.value.as_str()));
    }
}
