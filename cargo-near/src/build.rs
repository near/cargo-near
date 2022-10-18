use crate::abi::{AbiCompression, AbiFormat, AbiResult};
use crate::cargo::{manifest::CargoManifestPath, metadata::CrateMetadata};
use crate::{abi, util, BuildCommand};
use colored::Colorize;
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

    let mut pretty_abi_path = None;
    let mut min_abi_path = None;
    if !args.no_abi {
        let contract_abi = abi::generate_abi(&crate_metadata, args.doc, true)?;
        let AbiResult { path } = abi::write_to_file(
            &contract_abi,
            &crate_metadata,
            AbiFormat::Json,
            AbiCompression::NoOp,
        )?;
        pretty_abi_path.replace(util::copy(&path, &out_dir)?);

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
    }

    if let (true, Some(abi_path)) = (args.embed_abi, &min_abi_path) {
        cargo_args.extend(&["--features", "near-sdk/__abi-embed"]);
        build_env.push(("CARGO_NEAR_ABI_PATH", abi_path.to_str().unwrap()));
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
        wasm_artifact
            .path
            .display()
            .to_string()
            .bright_yellow()
            .bold(),
    )];
    if let Some(abi_path) = pretty_abi_path {
        messages.push(("ABI", abi_path.display().to_string().yellow().bold()));
    }
    if let Some(abi_path) = min_abi_path {
        messages.push((
            "Embedded ABI",
            abi_path.display().to_string().yellow().bold(),
        ));
    }

    let max_width = messages.iter().map(|(h, _)| h.len()).max().unwrap();
    for (header, message) in messages {
        eprintln!("     - {:>width$}: {}", header, message, width = max_width);
    }

    Ok(())
}
