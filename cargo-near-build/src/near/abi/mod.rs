use camino::Utf8PathBuf;
use colored::Colorize;

use crate::{
    pretty_print,
    types::{
        cargo::{manifest_path::ManifestPath, metadata::CrateMetadata},
        near::abi as abi_types,
    },
};

use crate::types::near::build::input::ColorPreference;
pub mod generate;

pub fn build(args: abi_types::Opts) -> eyre::Result<()> {
    let color = args.color.unwrap_or(ColorPreference::Auto);
    color.apply();

    let crate_metadata = pretty_print::handle_step("Collecting cargo project metadata...", || {
        let manifest_path: Utf8PathBuf = if let Some(manifest_path) = args.manifest_path {
            manifest_path
        } else {
            "Cargo.toml".into()
        };
        CrateMetadata::collect(ManifestPath::try_from(manifest_path)?, args.no_locked)
    })?;

    let out_dir = crate_metadata.resolve_output_dir(args.out_dir.map(Into::into))?;

    let format = if args.compact_abi {
        abi_types::Format::JsonMin
    } else {
        abi_types::Format::Json
    };
    let contract_abi = generate::procedure(
        &crate_metadata,
        args.no_locked,
        !args.no_doc,
        false,
        &[],
        color,
    )?;
    let abi_types::Result { path } = write_to_file(
        &contract_abi,
        &crate_metadata,
        format,
        abi_types::Compression::NoOp,
    )?;

    let abi_path = crate::fs::copy(&path, &out_dir)?;

    pretty_print::success("ABI Successfully Generated!");
    eprintln!("     - ABI: {}", abi_path.to_string().yellow().bold());

    Ok(())
}

pub fn write_to_file(
    contract_abi: &near_abi::AbiRoot,
    crate_metadata: &CrateMetadata,
    format: abi_types::Format,
    compression: abi_types::Compression,
) -> eyre::Result<abi_types::Result> {
    let near_abi_serialized = match format {
        abi_types::Format::Json => serde_json::to_vec_pretty(&contract_abi)?,
        abi_types::Format::JsonMin => serde_json::to_vec(&contract_abi)?,
    };
    let near_abi_compressed = match compression {
        abi_types::Compression::NoOp => near_abi_serialized,
        abi_types::Compression::Zstd => zstd::encode_all(
            near_abi_serialized.as_slice(),
            *zstd::compression_level_range().end(),
        )?,
    };

    let out_path_abi = crate_metadata.target_directory.join(format!(
        "{}_abi.{}",
        crate_metadata.formatted_package_name(),
        abi_types::file_extension(format, compression)
    ));
    std::fs::write(&out_path_abi, near_abi_compressed)?;

    Ok(abi_types::Result { path: out_path_abi })
}
