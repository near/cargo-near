use std::{ffi::OsStr, io::BufRead, path::PathBuf, process::Command};

use eyre::WrapErr;

const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

pub fn wasm32_exists() -> bool {
    let result = get_rustc_wasm32_unknown_unknown_target_libdir();

    match result {
        Ok(wasm32_target_libdir_path) => {
            if wasm32_target_libdir_path.exists() {
                log::info!(
                    "Found {COMPILATION_TARGET} in {:?}",
                    wasm32_target_libdir_path
                );
                true
            } else {
                log::info!(
                    "Failed to find {COMPILATION_TARGET} in {:?}",
                    wasm32_target_libdir_path
                );
                false
            }
        }
        Err(_) => {
            log::error!("Some error in getting the target libdir, trying rustup..");

            invoke_rustup(["target", "list", "--installed"])
                .map(|stdout| {
                    stdout
                        .lines()
                        .any(|target| target.as_ref().map_or(false, |t| t == COMPILATION_TARGET))
                })
                .is_ok()
        }
    }
}

fn get_rustc_wasm32_unknown_unknown_target_libdir() -> eyre::Result<PathBuf> {
    let command = Command::new("rustc")
        .args(["--target", COMPILATION_TARGET, "--print", "target-libdir"])
        .output()?;

    if command.status.success() {
        Ok(String::from_utf8(command.stdout)?.trim().into())
    } else {
        eyre::bail!(
            "Getting rustc's wasm32-unknown-unknown target wasn't successful. Got {}",
            command.status,
        )
    }
}

fn invoke_rustup<I, S>(args: I) -> eyre::Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let rustup = std::env::var("RUSTUP").unwrap_or_else(|_| "rustup".to_string());

    let mut cmd = Command::new(rustup);
    cmd.args(args);

    log::info!("Invoking rustup: {:?}", cmd);

    let child = cmd
        .stdout(std::process::Stdio::piped())
        .spawn()
        .wrap_err_with(|| format!("Error executing `{:?}`", cmd))?;

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        eyre::bail!(
            "`{:?}` failed with exit code: {:?}",
            cmd,
            output.status.code()
        );
    }
}
