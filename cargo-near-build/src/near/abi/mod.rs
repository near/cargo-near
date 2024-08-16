use crate::types::{
    cargo::metadata::CrateMetadata,
    near::abi::{abi_file_extension, AbiCompression, AbiFormat, AbiResult},
};

pub mod generate;

// TODO: make non-pub
pub fn write_to_file(
    contract_abi: &near_abi::AbiRoot,
    crate_metadata: &CrateMetadata,
    format: AbiFormat,
    compression: AbiCompression,
) -> eyre::Result<AbiResult> {
    let near_abi_serialized = match format {
        AbiFormat::Json => serde_json::to_vec_pretty(&contract_abi)?,
        AbiFormat::JsonMin => serde_json::to_vec(&contract_abi)?,
    };
    let near_abi_compressed = match compression {
        AbiCompression::NoOp => near_abi_serialized,
        AbiCompression::Zstd => zstd::encode_all(
            near_abi_serialized.as_slice(),
            *zstd::compression_level_range().end(),
        )?,
    };

    let out_path_abi = crate_metadata.target_directory.join(format!(
        "{}_abi.{}",
        crate_metadata.formatted_package_name(),
        abi_file_extension(format, compression)
    ));
    std::fs::write(&out_path_abi, near_abi_compressed)?;

    Ok(AbiResult { path: out_path_abi })
}
