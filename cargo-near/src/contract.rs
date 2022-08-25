use super::{abi, util, BuildCommand};
use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use std::io::BufRead;

const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

pub(crate) fn build(args: BuildCommand) -> anyhow::Result<()> {
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
        .map(|out_dir| {
            out_dir.canonicalize().map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => {
                    anyhow::anyhow!("output directory `{}` does not exist", out_dir.display())
                }
                _ => err.into(),
            })
        })
        .transpose()?;

    let mut cargo_args = vec!["--target", COMPILATION_TARGET];
    if args.release {
        cargo_args.push("--release");
    }

    let wasm_artifact = if args.embed_abi {
        let abi::AbiResult {
            path: abi_file_path,
        } = abi::write_to_file(&crate_metadata, args.doc)?;

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
    if let Some(out_dir) = out_dir {
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
