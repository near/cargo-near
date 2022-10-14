use crate::cargo::manifest::CargoManifestPath;
use anyhow::{Context, Result};
use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::Message;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
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
            eprintln!("{}", m);
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
    // Ensure that cargo uses color output for piped stderr.
    cmd.args(["--color", "always"]);

    log::info!("Invoking cargo: {:?}", cmd);

    let mut child = cmd
        // capture the stdout to return from this function as bytes
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context(format!("Error executing `{:?}`", cmd))?;
    let mut child_stdout = child
        .stdout
        .take()
        .context("could not attach to child stdout")?;
    let child_stderr = child
        .stderr
        .take()
        .context("could not attach to child stderr")?;

    // stdout and stderr have to be processed concurrently to not block the process from progressing
    let thread_stdout = thread::spawn(move || {
        let mut result = Vec::new();
        child_stdout
            .read_to_end(&mut result)
            .expect("failed to read cargo stdout");
        result
    });
    let thread_stderr = thread::spawn(move || {
        let stderr_reader = BufReader::new(child_stderr);
        let stderr_lines = stderr_reader.lines();
        for line in stderr_lines {
            eprintln!("  {}", line.expect("failed to read cargo stderr"));
        }
    });

    let result = thread_stdout.join().expect("failed to join stdout thread");
    thread_stderr.join().expect("failed to join stderr thread");

    let output = child.wait()?;

    if output.success() {
        Ok(result)
    } else {
        print_cargo_errors(result);
        anyhow::bail!("`{:?}` failed with exit code: {:?}", cmd, output.code());
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

pub(crate) fn extract_abi_entries(
    dylib_path: &Path,
) -> anyhow::Result<Vec<near_abi::__private::ChunkedAbiEntry>> {
    let dylib_file_contents = fs::read(&dylib_path)?;
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
        .map(|sym_name| sym_name.to_string())
        .collect::<HashSet<_>>();
    if near_abi_symbols.is_empty() {
        anyhow::bail!("No NEAR ABI symbols found in the dylib");
    }
    log::debug!("Detected NEAR ABI symbols: {:?}", &near_abi_symbols);

    let mut entries = vec![];
    unsafe {
        let lib = libloading::Library::new(dylib_path)?;
        for symbol in near_abi_symbols {
            let entry: libloading::Symbol<fn() -> near_abi::__private::ChunkedAbiEntry> =
                lib.get(symbol.as_bytes())?;
            entries.push(entry().clone());
        }
    }
    Ok(entries)
}
