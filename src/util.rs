use anyhow::{Context, Result};
use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::Message;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cargo::manifest::CargoManifestPath;

const fn dylib_extension() -> &'static str {
    #[cfg(target_os = "linux")]
    return "so";

    #[cfg(target_os = "macos")]
    return "dylib";

    #[cfg(target_os = "windows")]
    return "dll";

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    compile_error!("Unsupported platform");
}

fn print_cargo_errors(output: Vec<u8>) {
    let reader = std::io::BufReader::new(output.as_slice());
    let messages = Message::parse_stream(reader)
        .map(|m| m.unwrap())
        .collect::<Vec<_>>();
    messages
        .into_iter()
        .filter_map(|m| match m {
            cargo_metadata::Message::CompilerMessage(message)
                if message.message.level == DiagnosticLevel::Error =>
            {
                message.message.rendered
            }
            _ => None,
        })
        .for_each(|m| {
            println!("{}", m);
        });
}

/// Invokes `cargo` with the subcommand `command`, the supplied `args` and set `env` variables.
///
/// If `working_dir` is set, cargo process will be spawned in the specified directory.
///
/// Returns execution standard output as a byte array.
pub(crate) fn invoke_cargo<I, S, P>(
    command: &str,
    args: I,
    working_dir: Option<P>,
    env: Vec<(&str, &str)>,
) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr>,
    P: AsRef<Path>,
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo);

    env.iter().for_each(|(env_key, env_val)| {
        cmd.env(env_key, env_val);
    });

    if let Some(path) = working_dir {
        log::debug!("Setting cargo working dir to '{}'", path.as_ref().display());
        cmd.current_dir(path);
    }

    cmd.arg(command);
    cmd.args(args);

    log::info!("Invoking cargo: {:?}", cmd);

    let child = cmd
        // capture the stdout to return from this function as bytes
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context(format!("Error executing `{:?}`", cmd))?;
    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        print_cargo_errors(output.stdout);
        anyhow::bail!(
            "`{:?}` failed with exit code: {:?}",
            cmd,
            output.status.code()
        );
    }
}

fn build_cargo_project(manifest_path: &CargoManifestPath) -> anyhow::Result<Vec<Message>> {
    let output = invoke_cargo(
        "build",
        &["--release", "--message-format=json"],
        manifest_path.directory().ok(),
        vec![],
    )?;

    let reader = std::io::BufReader::new(output.as_slice());
    Ok(Message::parse_stream(reader).map(|m| m.unwrap()).collect())
}

/// Builds the cargo project with manifest located at `manifest_path` and returns the path to the generated dynamic lib.
pub(crate) fn compile_dylib_project(manifest_path: &CargoManifestPath) -> anyhow::Result<PathBuf> {
    let messages = build_cargo_project(manifest_path)?;
    // We find the last compiler artifact message which should contain information about the
    // resulting dylib file
    let compile_artifact = messages
        .iter()
        .filter_map(|m| match m {
            cargo_metadata::Message::CompilerArtifact(artifact) => Some(artifact),
            _ => None,
        })
        .last()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Cargo failed to produce any compilation artifacts. \
                 Please check that your project contains a NEAR smart contract."
            )
        })?;
    // The project could have generated many auxiliary files, we are only interested in
    // dylib files with a specific (platform-dependent) extension
    let dylib_files = compile_artifact
        .filenames
        .iter()
        .cloned()
        .filter(|f| {
            f.extension()
                .map(|e| e == dylib_extension())
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    match dylib_files.as_slice() {
        [] => Err(anyhow::anyhow!(
            "Compilation resulted in no '.{}' target files. \
                 Please check that your project contains a NEAR smart contract.",
            dylib_extension()
        )),
        [file] => Ok(file.to_owned().into_std_path_buf()),
        _ => Err(anyhow::anyhow!(
            "Compilation resulted in more than one '.{}' target file: {:?}",
            dylib_extension(),
            dylib_files
        )),
    }
}
