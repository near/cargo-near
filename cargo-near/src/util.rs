use crate::cargo::manifest::CargoManifestPath;
use anyhow::{Context, Result};
use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::Message;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) const fn dylib_extension() -> &'static str {
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
    Message::parse_stream(reader)
        .map(|m| m.unwrap())
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
    I: IntoIterator<Item = S>,
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

pub(crate) fn invoke_rustup<I, S>(args: I) -> anyhow::Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let rustup = env::var("RUSTUP").unwrap_or_else(|_| "rustup".to_string());

    let mut cmd = Command::new(rustup);
    cmd.args(args);

    log::info!("Invoking rustup: {:?}", cmd);

    let child = cmd
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context(format!("Error executing `{:?}`", cmd))?;

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        anyhow::bail!(
            "`{:?}` failed with exit code: {:?}",
            cmd,
            output.status.code()
        );
    }
}

pub struct CompilationArtifact {
    pub path: PathBuf,
    pub fresh: bool,
}

/// Builds the cargo project with manifest located at `manifest_path` and returns the path to the generated artifact.
pub(crate) fn compile_project(
    manifest_path: &CargoManifestPath,
    args: &[&str],
    env: Vec<(&str, &str)>,
    artifact_extension: &str,
) -> anyhow::Result<CompilationArtifact> {
    let stdout = invoke_cargo(
        "build",
        [&["--message-format=json"], args].concat(),
        manifest_path.directory().ok(),
        env,
    )?;
    let reader = std::io::BufReader::new(&*stdout);
    let messages: Vec<_> = Message::parse_stream(reader).collect::<Result<_, _>>()?;

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
                .map(|e| e == artifact_extension)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    match dylib_files.as_slice() {
        [] => Err(anyhow::anyhow!(
            "Compilation resulted in no '.{}' target files. \
                 Please check that your project contains a NEAR smart contract.",
            artifact_extension
        )),
        [file] => Ok(CompilationArtifact {
            path: file.to_owned().into_std_path_buf(),
            fresh: !compile_artifact.fresh,
        }),
        _ => Err(anyhow::anyhow!(
            "Compilation resulted in more than one '.{}' target file: {:?}",
            artifact_extension,
            dylib_files
        )),
    }
}
