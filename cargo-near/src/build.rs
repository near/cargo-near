use crate::abi::AbiResult;
use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::{abi, util, BuildCommand};
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

    let out_dir = util::force_canonicalize_dir(
        &args
            .out_dir
            .unwrap_or_else(|| crate_metadata.target_directory.clone()),
    )?;

    let mut build_env = vec![("RUSTFLAGS", "-C link-arg=-s")];
    let mut cargo_args = vec!["--target", COMPILATION_TARGET];
    if args.release {
        cargo_args.push("--release");
    }

    let mut abi_path = None;
    if !args.no_abi {
        let AbiResult { path } = abi::write_to_file(&crate_metadata, args.doc)?;
        abi_path.replace(util::copy(&path, &out_dir)?);
    }

    let mut wasm_artifact = if let Some(ref abi_path) = abi_path {
        cargo_args.extend(&["--features", "near-sdk/__abi-embed"]);
        build_env.push(("CARGO_NEAR_ABI_PATH", abi_path.to_str().unwrap()));

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

    wasm_artifact.path = util::copy(&wasm_artifact.path, &out_dir)?;

    // todo! if we embedded, check that the binary exports the __contract_abi symbol
    println!("Contract Successfully Built!");
    println!("   - Binary: {}", wasm_artifact.path.display());
    if let Some(abi_path) = abi_path {
        println!("   -    ABI: {}", abi_path.display());
    }

    Ok(())
}
