use std::{ffi::OsStr, marker::PhantomData, process::Command, thread};

use camino::Utf8Path;
use cargo_metadata::{Artifact, Message};
use eyre::{ContextCompat, WrapErr};
use std::io::BufRead;

use crate::pretty_print;
use crate::types::near::build::input::ColorPreference;
use crate::types::{cargo::manifest_path::ManifestPath, near::build::output::CompilationArtifact};

use super::ArtifactType;

/// Builds the cargo project with manifest located at `manifest_path` and returns the path to the generated artifact.
pub fn run<T>(
    manifest_path: &ManifestPath,
    args: &[&str],
    env: Vec<(&str, &str)>,
    hide_warnings: bool,
    color: ColorPreference,
) -> eyre::Result<CompilationArtifact<T>>
where
    T: ArtifactType,
{
    let final_env = {
        // Convert to owned strings to allow modification
        let mut env: Vec<(String, String)> = env
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        if hide_warnings {
            // Check if RUSTFLAGS already exists and append -Awarnings
            if let Some(pos) = env
                .iter()
                .position(|(k, _)| k == crate::env_keys::RUSTFLAGS)
            {
                env[pos].1.push_str(" -Awarnings");
            } else {
                env.push((
                    crate::env_keys::RUSTFLAGS.to_string(),
                    "-Awarnings".to_string(),
                ));
            }
        }
        env
    };

    let removed_env = [crate::env_keys::CARGO_ENCODED_RUSTFLAGS];

    let artifacts = invoke_cargo(
        "build",
        [&["--message-format=json-render-diagnostics"], args].concat(),
        manifest_path.directory().ok(),
        final_env.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        &removed_env,
        color,
    )?;

    // We find the last compiler artifact message which should contain information about the
    // resulting dylib file
    let compile_artifact = artifacts.last().wrap_err(
        "Cargo failed to produce any compilation artifacts. \
                 Please check that your project contains a NEAR smart contract.",
    )?;
    // The project could have generated many auxiliary files, we are only interested in
    // dylib files with a specific (platform-dependent) extension
    let dylib_files = compile_artifact
        .filenames
        .iter()
        .filter(|f| {
            f.extension()
                .map(|e| e == <T as ArtifactType>::extension())
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    let mut dylib_files_iter = Vec::into_iter(dylib_files);
    match (dylib_files_iter.next(), dylib_files_iter.next()) {
        (None, None) => eyre::bail!(
            "Compilation resulted in no '.{}' target files. \
                 Please check that your project contains a NEAR smart contract.",
            <T as ArtifactType>::extension(),
        ),
        (Some(path), None) => Ok(CompilationArtifact {
            path,
            fresh: !compile_artifact.fresh,
            from_docker: false,
            builder_version_info: None,
            artifact_type: PhantomData,
        }),
        _ => eyre::bail!(
            "Compilation resulted in more than one '.{}' target file: {:?}",
            <T as ArtifactType>::extension(),
            dylib_files_iter.as_slice()
        ),
    }
}
/// Invokes `cargo` with the subcommand `command`, the supplied `args` and set `env` variables.
///
/// If `working_dir` is set, cargo process will be spawned in the specified directory.
///
/// Returns execution standard output as a byte array.
fn invoke_cargo<A, P, E, S, EK, EV>(
    command: &str,
    args: A,
    working_dir: Option<P>,
    env: E,
    removed_env: &[&str],
    color: ColorPreference,
) -> eyre::Result<Vec<Artifact>>
where
    A: IntoIterator<Item = S>,
    P: AsRef<Utf8Path>,
    E: IntoIterator<Item = (EK, EV)>,
    S: AsRef<OsStr>,
    EK: AsRef<OsStr>,
    EV: AsRef<OsStr>,
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo);

    for key in removed_env {
        cmd.env_remove(key);
    }
    cmd.envs(env);

    if let Some(path) = working_dir {
        let path = crate::fs::force_canonicalize_dir(path.as_ref())?;
        cmd.current_dir(path);
    }

    cmd.arg(command);
    cmd.args(args);

    match color {
        ColorPreference::Auto => cmd.args(["--color", "auto"]),
        ColorPreference::Always => cmd.args(["--color", "always"]),
        ColorPreference::Never => cmd.args(["--color", "never"]),
    };

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Invoking cargo:\n{}",
        pretty_print::indent_payload(&format!("{cmd:#?}"))
    );

    let mut child = cmd
        // capture the stdout to return from this function as bytes
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err_with(|| format!("Error executing `{cmd:?}`"))?;
    let child_stdout = child
        .stdout
        .take()
        .wrap_err("could not attach to child stdout")?;
    let child_stderr = child
        .stderr
        .take()
        .wrap_err("could not attach to child stderr")?;

    // stdout and stderr have to be processed concurrently to not block the process from progressing
    let thread_stdout = thread::spawn(move || -> eyre::Result<_, std::io::Error> {
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
                            eprintln!(" │ {line}");
                        }
                    }
                }
                _ => {}
            };
        }

        Ok(artifacts)
    });
    let thread_stderr = thread::spawn(move || {
        let stderr_reader = std::io::BufReader::new(child_stderr);
        let stderr_lines = stderr_reader.lines();
        for line in stderr_lines {
            eprintln!(" │ {}", line.expect("failed to read cargo stderr"));
        }
    });

    let result = thread_stdout.join().expect("failed to join stdout thread");
    thread_stderr.join().expect("failed to join stderr thread");

    let output = child.wait()?;

    if output.success() {
        Ok(result?)
    } else {
        eyre::bail!("`{:?}` failed with exit code: {:?}", cmd, output.code());
    }
}
