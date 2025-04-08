use camino::Utf8PathBuf;

use crate::types::cargo::metadata::CrateMetadata;

pub mod abi;
pub mod build;
#[cfg(feature = "build_script")]
pub mod build_extended;
#[cfg(feature = "docker")]
pub mod docker_build;

pub const EXPECTED_WASM_EXTENSION: &str = "wasm";

pub struct OutputPaths {
    pub out_dir: Utf8PathBuf,
    pub wasm_file: Utf8PathBuf,
}

impl OutputPaths {
    pub fn new(
        crate_metadata: &CrateMetadata,
        cli_override: Option<Utf8PathBuf>,
    ) -> eyre::Result<Self> {
        let out_dir = crate_metadata.resolve_output_dir(cli_override)?;

        let filename = Self::wasm_filename(crate_metadata);
        let result = Self {
            out_dir: out_dir.clone(),
            wasm_file: out_dir.join(filename),
        };
        Ok(result)
    }

    fn wasm_filename(crate_metadata: &CrateMetadata) -> String {
        format!(
            "{}.{}",
            crate_metadata.formatted_package_name(),
            EXPECTED_WASM_EXTENSION
        )
    }

    fn abi_filename(
        crate_metadata: &CrateMetadata,
        format: abi::Format,
        compression: abi::Compression,
    ) -> String {
        format!(
            "{}_abi.{}",
            crate_metadata.formatted_package_name(),
            abi::file_extension(format, compression)
        )
    }

    pub fn intermediate_abi_file(
        crate_metadata: &CrateMetadata,
        format: abi::Format,
        compression: abi::Compression,
    ) -> Utf8PathBuf {
        crate_metadata.target_directory.join(Self::abi_filename(
            crate_metadata,
            format,
            compression,
        ))
    }
}
