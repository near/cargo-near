use eyre::{Context, ContextCompat};

use crate::types::{cargo::manifest_path::ManifestPath, near::build::input::Opts};

/// Return value: [`Result::Ok`] is path to the wasm artifact obtained.
pub fn run(opts: Opts) -> eyre::Result<camino::Utf8PathBuf> {
    let command = {
        let mut cmd = std::process::Command::new("cargo");

        let workdir = ManifestPath::get_manifest_workdir(opts.manifest_path.clone())?;

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
            "error running a build command with `cargo near ...`\n\
            NOTE: if `cargo-near` CLI is not installed, see https://github.com/near/cargo-near?tab=readme-ov-file#installation\n\
            \n\
            Original command output:\n\
            {}",
            output_string
        ));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let result_line = stderr
        .lines()
        .filter(|x| x.starts_with(RESULT_PREFIX))
        .next_back();

    let out_path = result_line
        .wrap_err(format!(
            "a line starting with `{}` not found!",
            RESULT_PREFIX
        ))?
        .strip_prefix(RESULT_PREFIX)
        .expect("always starts with expected prefix");

    Ok(camino::Utf8PathBuf::from(out_path.to_string()))
}
