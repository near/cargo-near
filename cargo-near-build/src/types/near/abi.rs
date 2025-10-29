use crate::types::near::build::input::ColorPreference;

#[derive(Debug, Default)]
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
    /// Activate additional cargo features during ABI generation
    pub features: Option<String>,
    /// Coloring: auto, always, never
    pub color: Option<ColorPreference>,
}

/// ABI generation result.
pub struct Result {
    /// Path to the resulting ABI file.
    pub path: camino::Utf8PathBuf,
}

#[derive(Clone, Debug)]
pub enum Format {
    Json,
    JsonMin,
}

#[derive(Clone, Debug)]
pub enum Compression {
    NoOp,
    Zstd,
}

pub(super) fn file_extension(format: Format, compression: Compression) -> &'static str {
    match compression {
        Compression::NoOp => match format {
            Format::Json | Format::JsonMin => "json",
        },
        Compression::Zstd => "zst",
    }
}
