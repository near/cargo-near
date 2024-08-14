use std::collections::{BTreeMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::process::Command;
use std::thread;

use camino::Utf8Path;
use cargo_metadata::{Artifact, Message};
use cargo_near_build::cargo_native::ArtifactType;
use cargo_near_build::types::cargo::manifest_path::ManifestPath;
use cargo_near_build::types::near::{CompilationArtifact, VersionMismatch};
use color_eyre::eyre::{ContextCompat, WrapErr};

use cargo_near_build::types::color_preference::ColorPreference;

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
    color: ColorPreference,
) -> color_eyre::eyre::Result<Vec<Artifact>>
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

    cmd.envs(env);

    if let Some(path) = working_dir {
        let path = cargo_near_build::fs::force_canonicalize_dir(path.as_ref())?;
        log::debug!("Setting cargo working dir to '{}'", path);
        cmd.current_dir(path);
    }

    cmd.arg(command);
    cmd.args(args);

    match color {
        ColorPreference::Auto => cmd.args(["--color", "auto"]),
        ColorPreference::Always => cmd.args(["--color", "always"]),
        ColorPreference::Never => cmd.args(["--color", "never"]),
    };

    log::info!("Invoking cargo: {:?}", cmd);

    let mut child = cmd
        // capture the stdout to return from this function as bytes
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err_with(|| format!("Error executing `{:?}`", cmd))?;
    let child_stdout = child
        .stdout
        .take()
        .wrap_err("could not attach to child stdout")?;
    let child_stderr = child
        .stderr
        .take()
        .wrap_err("could not attach to child stderr")?;

    // stdout and stderr have to be processed concurrently to not block the process from progressing
    let thread_stdout = thread::spawn(move || -> color_eyre::eyre::Result<_, std::io::Error> {
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
    });
    let thread_stderr = thread::spawn(move || {
        let stderr_reader = BufReader::new(child_stderr);
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
        color_eyre::eyre::bail!("`{:?}` failed with exit code: {:?}", cmd, output.code());
    }
}

/// Builds the cargo project with manifest located at `manifest_path` and returns the path to the generated artifact.
pub(crate) fn compile_project<T>(
    manifest_path: &ManifestPath,
    args: &[&str],
    mut env: Vec<(&str, &str)>,
    hide_warnings: bool,
    color: ColorPreference,
) -> color_eyre::eyre::Result<CompilationArtifact<T>>
where
    T: ArtifactType,
{
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

    let artifacts = invoke_cargo(
        "build",
        [&["--message-format=json-render-diagnostics"], args].concat(),
        manifest_path.directory().ok(),
        final_env.iter(),
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
        (None, None) => color_eyre::eyre::bail!(
            "Compilation resulted in no '.{}' target files. \
                 Please check that your project contains a NEAR smart contract.",
            <T as ArtifactType>::extension(),
        ),
        (Some(path), None) => Ok(CompilationArtifact {
            path,
            fresh: !compile_artifact.fresh,
            from_docker: false,
            cargo_near_version_mismatch: VersionMismatch::None,
            artifact_type: PhantomData,
        }),
        _ => color_eyre::eyre::bail!(
            "Compilation resulted in more than one '.{}' target file: {:?}",
            <T as ArtifactType>::extension(),
            dylib_files_iter.as_slice()
        ),
    }
}

pub(crate) fn extract_abi_entries(
    dylib_path: &Utf8Path,
) -> color_eyre::eyre::Result<Vec<near_abi::__private::ChunkedAbiEntry>> {
    let dylib_file_contents = fs::read(dylib_path)?;
    let object = symbolic_debuginfo::Object::parse(&dylib_file_contents)?;
    log::debug!(
        "A dylib was built at {:?} with format {} for architecture {}",
        &dylib_path,
        &object.file_format(),
        &object.arch()
    );
    let near_abi_symbols = object
        .symbols()
        .flat_map(|sym| sym.name)
        .filter(|sym_name| sym_name.starts_with("__near_abi_"))
        .collect::<HashSet<_>>();
    if near_abi_symbols.is_empty() {
        color_eyre::eyre::bail!("No NEAR ABI symbols found in the dylib");
    }
    log::debug!("Detected NEAR ABI symbols: {:?}", &near_abi_symbols);

    let mut entries = vec![];
    unsafe {
        let lib = libloading::Library::new(dylib_path)?;
        for symbol in near_abi_symbols {
            let entry: libloading::Symbol<extern "C" fn() -> (*const u8, usize)> =
                lib.get(symbol.as_bytes())?;
            let (ptr, len) = entry();
            let data = Vec::from_raw_parts(ptr as *mut _, len, len);
            match serde_json::from_slice(&data) {
                Ok(entry) => entries.push(entry),
                Err(err) => {
                    // unfortunately, we're unable to extract the raw error without Display-ing it first
                    let mut err_str = err.to_string();
                    if let Some((msg, rest)) = err_str.rsplit_once(" at line ") {
                        if let Some((line, col)) = rest.rsplit_once(" column ") {
                            if line.chars().all(|c| c.is_numeric())
                                && col.chars().all(|c| c.is_numeric())
                            {
                                err_str.truncate(msg.len());
                                err_str.shrink_to_fit();
                                color_eyre::eyre::bail!(err_str);
                            }
                        }
                    }
                    color_eyre::eyre::bail!(err);
                }
            };
        }
    }
    Ok(entries)
}
