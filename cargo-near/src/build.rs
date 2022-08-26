use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::{abi, util, BuildCommand};
use anyhow::Context;
use std::fs;
use std::io::BufRead;

const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

pub(crate) fn run(args: BuildCommand) -> anyhow::Result<()> {
    if !util::invoke_rustup(&["target", "list", "--installed"])?
        .lines()
        .any(|target| target.as_ref().map_or(false, |t| t == COMPILATION_TARGET))
    {
        anyhow::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
    }

    let crate_metadata = CrateMetadata::collect(CargoManifestPath::try_from(
        args.manifest_path.unwrap_or_else(|| "Cargo.toml".into()),
    )?)?;

    let out_dir = args
        .out_dir
        .unwrap_or(crate_metadata.target_directory.clone());
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create directory `{}`", out_dir.display()))?;
    let out_dir = out_dir
        .canonicalize()
        .with_context(|| format!("failed to access output directory `{}`", out_dir.display()))?;

    let mut build_env = vec![("RUSTFLAGS", "-C link-arg=-s")];
    let mut cargo_args = vec!["--target", COMPILATION_TARGET];
    if args.release {
        cargo_args.push("--release");
    }

    let abi::AbiResult {
        path: abi_file_path,
    } = abi::write_to_file(&crate_metadata, args.doc)?;

    let wasm_artifact = if args.embed_abi {
        cargo_args.extend(&["--features", "near-sdk/__abi-embed"]);
        build_env.push(("CARGO_NEAR_ABI_PATH", abi_file_path.to_str().unwrap()));

        util::compile_project(
            &crate_metadata.manifest_path,
            &cargo_args,
            build_env,
            "wasm",
        )?
    } else {
        util::compile_project(
            &crate_metadata.manifest_path,
            &cargo_args,
            build_env,
            "wasm",
        )?
    };

    let mut out_path = wasm_artifact.path;
    if out_path
        .parent()
        .map_or(false, |out_path| out_dir != out_path)
    {
        let new_out_path = out_dir.join(out_path.file_name().unwrap());
        std::fs::copy(&out_path, &new_out_path).with_context(|| {
            format!(
                "failed to copy `{}` to `{}`",
                out_path.display(),
                new_out_path.display(),
            )
        })?;
        out_path = new_out_path;
    }

    // todo! if we embedded, check that the binary exports the __contract_abi symbol
    println!("Contract successfully built at {}", out_path.display());

    Ok(())
}
