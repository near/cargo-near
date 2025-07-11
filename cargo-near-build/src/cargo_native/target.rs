use std::{ffi::OsStr, io::BufRead, path::PathBuf, process::Command};

use eyre::WrapErr;

use crate::{env_keys::RUSTUP_TOOLCHAIN, pretty_print};

pub const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

pub fn wasm32_exists(override_toolchain: Option<String>) -> bool {
    let result = get_rustc_wasm32_unknown_unknown_target_libdir(override_toolchain.clone());

    match result {
        Ok(wasm32_target_libdir_path) => {
            if wasm32_target_libdir_path.exists() {
                tracing::info!(
                    target: "near_teach_me",
                    parent: &tracing::Span::none(),
                    "Found {COMPILATION_TARGET} in {:?}",
                    wasm32_target_libdir_path
                );
                true
            } else {
                tracing::info!(
                    target: "near_teach_me",
                    parent: &tracing::Span::none(),
                    "Failed to find {COMPILATION_TARGET} in {:?}",
                    wasm32_target_libdir_path
                );
                false
            }
        }
        Err(_) => {
            tracing::error!("Some error in getting the target libdir, trying rustup..");

            invoke_rustup(["target", "list", "--installed"], override_toolchain)
                .map(|stdout| {
                    stdout
                        .lines()
                        .any(|target| target.as_ref().is_ok_and(|t| t == COMPILATION_TARGET))
                })
                .is_ok()
        }
    }
}

fn get_rustc_wasm32_unknown_unknown_target_libdir(
    override_toolchain: Option<String>,
) -> eyre::Result<PathBuf> {
    let mut command = Command::new("rustc");

    command.args(["--target", COMPILATION_TARGET, "--print", "target-libdir"]);
    if let Some(toolchain) = override_toolchain {
        command.env(RUSTUP_TOOLCHAIN, toolchain);
    }

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Command execution:\n{}",
        pretty_print::indent_payload(&format!("`{command:?}`").replace("\"", ""))
    );

    let output = command.output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().into())
    } else {
        eyre::bail!(
            "Getting rustc's wasm32-unknown-unknown target wasn't successful. Got {}",
            output.status,
        )
    }
}

fn invoke_rustup<I, S>(args: I, override_toolchain: Option<String>) -> eyre::Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let rustup = std::env::var("RUSTUP").unwrap_or_else(|_| "rustup".to_string());

    let mut cmd = Command::new(rustup);
    cmd.args(args);
    if let Some(toolchain) = override_toolchain {
        cmd.env(RUSTUP_TOOLCHAIN, toolchain);
    }

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Invoking rustup:\n{}",
        pretty_print::indent_payload(&format!("`{}`", format!("{cmd:?}").replace("\"", "")))
    );

    let child = cmd
        .stdout(std::process::Stdio::piped())
        .spawn()
        .wrap_err_with(|| format!("Error executing `{cmd:?}`"))?;

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
