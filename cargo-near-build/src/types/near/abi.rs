use camino::Utf8PathBuf;

use crate::types::color_preference;

#[derive(Default)]
pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    pub no_locked: bool,
    /// Include rustdocs in the ABI file
    pub no_doc: bool,
    /// Generate compact (minified) JSON
    pub compact_abi: bool,
    /// Copy final artifacts to this directory
    pub out_dir: Option<camino::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    pub manifest_path: Option<camino::Utf8PathBuf>,
    /// Coloring: auto, always, never
    pub color: Option<color_preference::ColorPreference>,
}

/// ABI generation result.
pub struct AbiResult {
    /// Path to the resulting ABI file.
    pub path: Utf8PathBuf,
}

#[derive(Clone, Debug)]
pub enum AbiFormat {
    Json,
    JsonMin,
}

#[derive(Clone, Debug)]
pub enum AbiCompression {
    NoOp,
    Zstd,
}

pub(crate) fn abi_file_extension(format: AbiFormat, compression: AbiCompression) -> &'static str {
    match compression {
        AbiCompression::NoOp => match format {
            AbiFormat::Json | AbiFormat::JsonMin => "json",
        },
        AbiCompression::Zstd => "zst",
    }
}
