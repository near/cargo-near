use std::env;
use std::ffi::OsStr;
use std::io::BufRead;
use std::process::Command;

use anyhow::Context;

use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};

use super::{abi, util, BuildCommand};

fn invoke_rustup<I, S>(args: I) -> anyhow::Result<Vec<u8>>
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

const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

pub(crate) fn build(args: BuildCommand) -> anyhow::Result<()> {
    if !invoke_rustup(&["target", "list", "--installed"])?
        .lines()
        .any(|target| target.as_ref().map_or(false, |t| t == COMPILATION_TARGET))
    {
        anyhow::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
    }

    let crate_metadata = CrateMetadata::collect(CargoManifestPath::try_from(
        args.manifest_path.unwrap_or_else(|| "Cargo.toml".into()),
    )?)?;

    let mut cargo_args = vec!["--target", COMPILATION_TARGET];
    if args.release {
        cargo_args.push("--release");
    }

    let wasm_artifact = if args.embed_abi {
        let abi::AbiResult {
            path: abi_file_path,
        } = abi::write_to_file(&crate_metadata)?;

        // todo! add compression.
        // todo! test differences between snappy and zstd

        cargo_args.extend(&["--features", "near-sdk/__abi-embed"]);

        util::compile_project(
            &crate_metadata.manifest_path,
            &cargo_args,
            vec![("CARGO_NEAR_ABI_PATH", abi_file_path.to_str().unwrap())],
            "wasm",
        )?
    } else {
        util::compile_project(&crate_metadata.manifest_path, &cargo_args, vec![], "wasm")?
    };

    let mut out_path = wasm_artifact.path;
    if let Some(out_dir) = args.out_dir.and_then(|o| o.canonicalize().ok()) {
        if out_dir != out_path.parent().unwrap() {
            let new_out_path = out_dir.join(out_path.file_name().unwrap());
            std::fs::copy(&out_path, &new_out_path)?;
            out_path = new_out_path;
        }
    }

    // todo! if we embedded, check that the binary exports the __contract_abi symbol
    println!("Contract successfully built at {}", out_path.display());

    Ok(())
}
