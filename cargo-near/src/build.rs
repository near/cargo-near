use crate::abi::{AbiCompression, AbiFormat, AbiResult};
use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::{abi, util, BuildCommand};
use colored::Colorize;
use near_abi::BuildInfo;
use sha2::{Digest, Sha256};
use std::io::BufRead;

const COMPILATION_TARGET: &str = "wasm32-unknown-unknown";

pub(crate) fn run(args: BuildCommand) -> anyhow::Result<()> {
    util::handle_step("Checking the host environment...", || {
        if !util::invoke_rustup(&["target", "list", "--installed"])?
            .lines()
            .any(|target| target.as_ref().map_or(false, |t| t == COMPILATION_TARGET))
        {
            anyhow::bail!("rust target `{}` is not installed", COMPILATION_TARGET);
        }
        Ok(())
    })?;

    let crate_metadata = util::handle_step("Collecting cargo project metadata...", || {
        CrateMetadata::collect(CargoManifestPath::try_from(
            args.manifest_path.unwrap_or_else(|| "Cargo.toml".into()),
        )?)
    })?;

    let out_dir = args
        .out_dir
        .map_or(Ok(crate_metadata.target_directory.clone()), |out_dir| {
            util::force_canonicalize_dir(&out_dir)
        })?;

    let mut build_env = vec![("RUSTFLAGS", "-C link-arg=-s")];
    let mut cargo_args = vec!["--target", COMPILATION_TARGET];
    if args.release {
        cargo_args.push("--release");
    }

    let mut abi = None;
    let mut min_abi_path = None;
    if !args.no_abi {
        let mut contract_abi = abi::generate_abi(&crate_metadata, args.doc, true)?;
        contract_abi.metadata.build = Some(BuildInfo {
            compiler: format!("rustc {}", rustc_version::version()?),
            builder: format!("cargo-near {}", env!("CARGO_PKG_VERSION")),
            image: None,
        });
        if args.embed_abi {
            let path = util::handle_step("Compressing ABI to be embedded..", || {
                let AbiResult { path } = abi::write_to_file(
                    &contract_abi,
                    &crate_metadata,
                    AbiFormat::JsonMin,
                    AbiCompression::Zstd,
                )?;
                anyhow::Ok(path)
            })?;
            min_abi_path.replace(util::copy(&path, &out_dir)?);
        }
        abi = Some(contract_abi);
    }

    if let (true, Some(abi_path)) = (args.embed_abi, &min_abi_path) {
        cargo_args.extend(&["--features", "near-sdk/__abi-embed"]);
        build_env.push(("CARGO_NEAR_ABI_PATH", abi_path.as_str()));
    }
    util::print_step("Building contract");
    let mut wasm_artifact = util::compile_project(
        &crate_metadata.manifest_path,
        &cargo_args,
        build_env,
        "wasm",
        false,
    )?;

    wasm_artifact.path = util::copy(&wasm_artifact.path, &out_dir)?;

    // todo! if we embedded, check that the binary exports the __contract_abi symbol
    util::print_success("Contract successfully built!");
    let mut messages = vec![(
        "Binary",
        wasm_artifact.path.to_string().bright_yellow().bold(),
    )];
    if let Some(mut abi) = abi {
        let mut hasher = Sha256::new();
        hasher.update(std::fs::read(&wasm_artifact.path)?);
        let hash = hasher.finalize();
        let hash = bs58::encode(hash).into_string();
        abi.metadata.wasm_hash = Some(hash);

        let AbiResult { path } =
            abi::write_to_file(&abi, &crate_metadata, AbiFormat::Json, AbiCompression::NoOp)?;
        let pretty_abi_path = util::copy(&path, &out_dir)?;
        messages.push(("ABI", pretty_abi_path.to_string().yellow().bold()));
    }
    if let Some(abi_path) = min_abi_path {
        messages.push(("Embedded ABI", abi_path.to_string().yellow().bold()));
    }

    let max_width = messages.iter().map(|(h, _)| h.len()).max().unwrap();
    for (header, message) in messages {
        eprintln!("     - {:>width$}: {}", header, message, width = max_width);
    }

    Ok(())
}
