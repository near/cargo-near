use eyre::{Context, ContextCompat};

use crate::types::{
    cargo::manifest_path::{ManifestPath, MANIFEST_FILE_NAME},
    near::build::input::Opts,
};

pub fn run(opts: Opts) -> eyre::Result<camino::Utf8PathBuf> {
    let command = {
        let mut cmd = std::process::Command::new("cargo");

        let workdir = {
            let manifest_path: camino::Utf8PathBuf =
                if let Some(manifest_path) = opts.manifest_path.clone() {
                    manifest_path
                } else {
                    MANIFEST_FILE_NAME.into()
                };
            let manifest_path = ManifestPath::try_from(manifest_path)?;
            manifest_path.directory()?.to_path_buf()
        };

        cmd.current_dir(workdir);
        cmd.args(opts.get_cli_command_for_lib_context().into_iter().skip(1));

        if let Some(override_cargo_target_dir) = opts.override_cargo_target_dir {
            cmd.env(
                crate::env_keys::CARGO_TARGET_DIR,
                &override_cargo_target_dir,
            );
        }
        if let Some(nep330_contract_path) = opts.override_nep330_contract_path {
            cmd.env(crate::env_keys::nep330::CONTRACT_PATH, nep330_contract_path);
        }
        if let Some(nep330_output_wasm_path) = opts.override_nep330_output_wasm_path {
            cmd.env(
                crate::env_keys::nep330::OUTPUT_WASM_PATH,
                nep330_output_wasm_path,
            );
        }
        cmd.env(crate::env_keys::COLOR_PREFERENCE_NO_COLOR, "true");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd
    };

    run_command(command)
}

const RESULT_PREFIX: &str = "     -                Binary: ";

fn run_command(mut command: std::process::Command) -> eyre::Result<camino::Utf8PathBuf> {
    let process = command
        .spawn()
        .wrap_err("could not spawn `cargo-near` process")?;

    let output = process
        .wait_with_output()
        .wrap_err("err waiting for `cargo-near` to finish")?;

    let output_string = {
        let mut output_string = String::new();
        output_string.push_str(&String::from_utf8_lossy(&output.stderr));
        output_string.push_str(&String::from_utf8_lossy(&output.stdout));
        output_string
    };

    if !output.status.success() {
        return Err(eyre::eyre!(
            "error running a build command with `cargo-near`:\n {}",
            output_string
        ))
        .wrap_err("`cargo-near` CLI not installed. See https://github.com/near/cargo-near?tab=readme-ov-file#installation");
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let result_line = stderr
        .lines()
        .filter(|x| x.starts_with(RESULT_PREFIX))
        .last();

    let out_path = result_line
        .wrap_err(format!(
            "a line starting with `{}` not found!",
            RESULT_PREFIX
        ))?
        .strip_prefix(RESULT_PREFIX)
        .expect("always starts with expected prefix");

    Ok(camino::Utf8PathBuf::from(out_path.to_string()))
}
