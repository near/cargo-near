use camino::Utf8PathBuf;

use crate::types::cargo::metadata::CrateMetadata;

#[cfg(feature = "build_internal")]
pub mod abi;
pub mod build;

pub mod build_extended;

#[cfg(feature = "docker")]
pub mod docker_build;

pub const EXPECTED_WASM_EXTENSION: &str = "wasm";

// TODO #F: uncomment for `build_external_extended` method
#[allow(unused)]
pub struct OutputPaths {
    out_dir: Utf8PathBuf,
    wasm_file: Utf8PathBuf,
}

impl OutputPaths {
    // TODO #F: uncomment for `build_external_extended` method
    #[allow(unused)]
    pub fn new(
        crate_metadata: &CrateMetadata,
        cli_override: Option<Utf8PathBuf>,
    ) -> eyre::Result<Self> {
        let out_dir = crate_metadata.resolve_output_dir(cli_override)?;

        let filename = Self::wasm_filename(crate_metadata);
        let wasm_file = out_dir.join(filename);
        assert!(
            out_dir.is_absolute(),
            "{out_dir} expected to be an absolute path"
        );
        assert!(
            wasm_file.is_absolute(),
            "{wasm_file} expected to be an absolute path"
        );
        let result = Self { out_dir, wasm_file };

        Ok(result)
    }
    // TODO #F: uncomment for `build_external_extended` method
    #[allow(unused)]
    pub fn get_wasm_file(&self) -> &Utf8PathBuf {
        &self.wasm_file
    }
    #[cfg(feature = "build_internal")]
    pub fn get_out_dir(&self) -> &Utf8PathBuf {
        &self.out_dir
    }

    fn wasm_filename(crate_metadata: &CrateMetadata) -> String {
        format!(
            "{}.{}",
            crate_metadata.formatted_package_name(),
            EXPECTED_WASM_EXTENSION
        )
    }

    #[cfg(feature = "build_internal")]
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

    #[cfg(feature = "build_internal")]
    pub(crate) fn intermediate_abi_file(
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
