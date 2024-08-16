use camino::Utf8PathBuf;
use cargo_near_build::near::abi::{self, write_to_file};
use cargo_near_build::pretty_print;
use cargo_near_build::types::cargo::manifest_path::ManifestPath;
use cargo_near_build::types::near::abi::{AbiCompression, AbiFormat, AbiResult};
use colored::Colorize;

use cargo_near_build::types::cargo::metadata::CrateMetadata;
use cargo_near_build::types::color_preference::ColorPreference;

pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    pub no_locked: bool,
    /// Include rustdocs in the ABI file
    pub no_doc: bool,
    /// Generate compact (minified) JSON
    pub compact_abi: bool,
    /// Copy final artifacts to this directory
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Coloring: auto, always, never
    pub color: Option<cargo_near_build::types::color_preference::ColorPreference>,
}

impl From<super::AbiCommand> for Opts {
    fn from(value: super::AbiCommand) -> Self {
        Self {
            no_locked: value.no_locked,
            no_doc: value.no_doc,
            compact_abi: value.compact_abi,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            color: value.color.map(Into::into),
        }
    }
}

pub fn run(args: Opts) -> near_cli_rs::CliResult {
    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        let manifest_path: Utf8PathBuf = if let Some(manifest_path) = args.manifest_path {
            manifest_path.into()
        } else {
            "Cargo.toml".into()
        };
        CrateMetadata::collect(ManifestPath::try_from(manifest_path)?, args.no_locked)
    })?;

    let out_dir = crate_metadata.resolve_output_dir(args.out_dir.map(Into::into))?;

    let format = if args.compact_abi {
        AbiFormat::JsonMin
    } else {
        AbiFormat::Json
    };
    let contract_abi = abi::generate::procedure(
        &crate_metadata,
        args.no_locked,
        !args.no_doc,
        false,
        &[],
        color,
    )?;
    let AbiResult { path } =
        write_to_file(&contract_abi, &crate_metadata, format, AbiCompression::NoOp)?;

    let abi_path = cargo_near_build::fs::copy(&path, &out_dir)?;

    pretty_print::success("ABI Successfully Generated!");
    eprintln!("     - ABI: {}", abi_path.to_string().yellow().bold());

    Ok(())
}
