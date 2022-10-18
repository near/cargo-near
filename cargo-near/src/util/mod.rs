use crate::cargo::manifest::CargoManifestPath;
use anyhow::{Context, Result};
use cargo_metadata::{Artifact, Message};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, ChildStdout, Command};
use std::{env, thread};

mod print;
pub(crate) use print::*;

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

/// Invokes `cargo` with the subcommand `command`, the supplied `args` and set `env` variables.
///
/// If `working_dir` is set, cargo process will be spawned in the specified directory.
///
/// Returns execution standard output as a byte array.
pub(crate) fn invoke_cargo_generic<
    Args,
    Arg,
    TPath,
    Env,
    EnvKey,
    EnvVal,
    StdoutFn,
    StdoutResult,
    StderrFn,
    StderrResult,
>(
    command: &str,
    args: Args,
    working_dir: Option<TPath>,
    env: Env,
    stdout_fn: StdoutFn,
    stderr_fn: StderrFn,
) -> Result<(StdoutResult, StderrResult)>
where
    Args: IntoIterator<Item = Arg>,
    Arg: AsRef<OsStr>,
    TPath: AsRef<Path>,
    Env: IntoIterator<Item = (EnvKey, EnvVal)>,
    EnvKey: AsRef<OsStr>,
    EnvVal: AsRef<OsStr>,
    StdoutFn: FnOnce(ChildStdout) -> Result<StdoutResult, std::io::Error> + Send + 'static,
    StdoutResult: Send + 'static,
    StderrFn: FnOnce(ChildStderr) -> Result<StderrResult, std::io::Error> + Send + 'static,
    StderrResult: Send + 'static,
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo);

    cmd.envs(env);

    if let Some(path) = working_dir {
        log::debug!("Setting cargo working dir to '{}'", path.as_ref().display());
        cmd.current_dir(path);
    }

    cmd.arg(command);
    cmd.args(args);
    // Ensure that cargo uses color output for piped stderr.
    cmd.args(["--color", "always"]);

    log::info!("Invoking cargo: {:?}", cmd);

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context(format!("Error executing `{:?}`", cmd))?;
    let child_stdout = child
        .stdout
        .take()
        .context("could not attach to child stdout")?;
    let child_stderr = child
        .stderr
        .take()
        .context("could not attach to child stderr")?;

    // stdout and stderr have to be processed concurrently to not block the process from progressing
    let thread_stdout =
        thread::spawn(move || -> Result<StdoutResult, std::io::Error> { stdout_fn(child_stdout) });
    let thread_stderr =
        thread::spawn(move || -> Result<StderrResult, std::io::Error> { stderr_fn(child_stderr) });

    let stdout_result = thread_stdout.join().expect("failed to join stdout thread");
    let stderr_result = thread_stderr.join().expect("failed to join stderr thread");

    let output = child.wait()?;

    if output.success() {
        Ok((stdout_result?, stderr_result?))
    } else {
        anyhow::bail!("`{:?}` failed with exit code: {:?}", cmd, output.code());
    }
}

/// Invokes `cargo` with the subcommand `command`, the supplied `args` and set `env` variables.
/// Expects `cargo` to output JSON message stream, so make sure to pass either
/// "--message-format=json-render-diagnostics" or "--message-format=json" in the `args`.
///
/// If `working_dir` is set, cargo process will be spawned in the specified directory.
///
/// Returns list of artifacts produced by `cargo`.
pub(crate) fn invoke_cargo_json<Args, Arg, TPath, Env, EnvKey, EnvVal>(
    command: &str,
    args: Args,
    working_dir: Option<TPath>,
    env: Env,
) -> Result<Vec<Artifact>>
where
    Args: IntoIterator<Item = Arg>,
    Arg: AsRef<OsStr>,
    TPath: AsRef<Path>,
    Env: IntoIterator<Item = (EnvKey, EnvVal)>,
    EnvKey: AsRef<OsStr>,
    EnvVal: AsRef<OsStr>,
{
    let (result, _) = invoke_cargo_generic(command, args, working_dir, env, stdout_fn, stderr_fn)?;

    fn stdout_fn(child_stdout: ChildStdout) -> Result<Vec<Artifact>, std::io::Error> {
        let mut artifacts = vec![];
        let stdout_reader = std::io::BufReader::new(child_stdout);
        for message in Message::parse_stream(stdout_reader) {
            match message? {
                Message::CompilerArtifact(artifact) => {
                    artifacts.push(artifact);
                }
                Message::CompilerMessage(message) => {
                    if let Some(msg) = message.message.rendered {
                        for line in msg.lines() {
                            eprintln!(" │ {}", line);
                        }
                    }
                }
                _ => {}
            };
        }

        Ok(artifacts)
    }

    fn stderr_fn(child_stderr: ChildStderr) -> Result<(), std::io::Error> {
        let stderr_reader = BufReader::new(child_stderr);
        let stderr_lines = stderr_reader.lines();
        for line in stderr_lines {
            eprintln!(" │ {}", line.expect("failed to read cargo stderr"));
        }

        Ok(())
    }

    Ok(result)
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
    mut env: Vec<(&str, &str)>,
    artifact_extension: &str,
    hide_warnings: bool,
) -> anyhow::Result<CompilationArtifact> {
    let mut final_env = BTreeMap::new();

    if hide_warnings {
        env.push(("RUSTFLAGS", "-Awarnings"));
    }

    for (key, value) in env {
        match key {
            "RUSTFLAGS" => {
                let rustflags: &mut String = final_env
                    .entry(key)
                    .or_insert_with(|| std::env::var(key).unwrap_or_default());
                if !rustflags.is_empty() {
                    rustflags.push(' ');
                }
                rustflags.push_str(value);
            }
            _ => {
                final_env.insert(key, value.to_string());
            }
        }
    }

    let artifacts = invoke_cargo_json(
        "build",
        [&["--message-format=json-render-diagnostics"], args].concat(),
        manifest_path.directory().ok(),
        final_env.iter(),
    )?;

    // We find the last compiler artifact message which should contain information about the
    // resulting dylib file
    let compile_artifact = artifacts.last().ok_or_else(|| {
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

/// Create the directory if it doesn't exist, and return the absolute path to it.
pub(crate) fn force_canonicalize_dir(dir: &Path) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create directory `{}`", dir.display()))?;
    dir.canonicalize()
        .with_context(|| format!("failed to access output directory `{}`", dir.display()))
}

/// Copy a file to a destination.
///
/// Does nothing if the destination is the same as the source to avoid truncating the file.
pub(crate) fn copy(from: &Path, to: &Path) -> anyhow::Result<PathBuf> {
    let out_path = to.join(from.file_name().unwrap());
    if from != out_path {
        fs::copy(&from, &out_path).with_context(|| {
            format!(
                "failed to copy `{}` to `{}`",
                from.display(),
                out_path.display(),
            )
        })?;
    }
    Ok(out_path)
}
