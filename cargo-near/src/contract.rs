use std::env;
use std::ffi::OsStr;
use std::fs;
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
    if let None = invoke_rustup(&["target", "list", "--installed"])?
        .lines()
        .find(|target| target.as_ref().map_or(false, |t| t == COMPILATION_TARGET))
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
            source_hash,
            path: abi_file_path,
        } = abi::write_to_file(&crate_metadata)?;

        // todo! add compression.
        // todo! test differences between snappy and zstd

        let mut ack_secret = [0; 8];
        getrandom::getrandom(&mut ack_secret).context("failed to generate ack secret")?;
        let mut ack_secret_hex = String::new();
        for b in ack_secret.iter() {
            ack_secret_hex.push_str(&format!("{:02x}", b));
        }

        let wasm_artifact = util::compile_project(
            &crate_metadata.manifest_path,
            &cargo_args,
            vec![
                ("RUSTC_WRAPPER", &std::env::args().next().unwrap()),
                ("CARGO_NEAR_ABI_SOURCE_HASH", &format!("{:x}", source_hash)),
                ("CARGO_NEAR_ABI_PATH", abi_file_path.to_str().unwrap()),
                ("CARGO_NEAR_ABI_ACK", &ack_secret_hex),
            ],
            "wasm",
        )?;

        if wasm_artifact.fresh {
            let ack_path = abi_file_path.with_extension("ack");
            let file = fs::read_to_string(&ack_path)?;
            if ack_secret_hex != file {
                anyhow::bail!("cargo-near was unable to embed the ABI");
            }
            fs::remove_file(ack_path)?;
        }

        wasm_artifact
    } else {
        util::compile_project(&crate_metadata.manifest_path, &cargo_args, vec![], "wasm")?
    };

    let mut out_path = wasm_artifact.path;
    if let Some(out_dir) = args.out_dir.map(|o| o.canonicalize().ok()).flatten() {
        if out_dir != out_path.parent().unwrap() {
            let new_out_path = out_dir.join(out_path.file_name().unwrap());
            std::fs::copy(&out_path, &new_out_path)?;
            out_path = new_out_path;
        }
    }

    println!("Contract successfully built at {}", out_path.display());

    Ok(())
}
