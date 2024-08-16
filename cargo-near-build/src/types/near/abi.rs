use camino::Utf8PathBuf;

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

pub fn abi_file_extension(format: AbiFormat, compression: AbiCompression) -> &'static str {
    match compression {
        AbiCompression::NoOp => match format {
            AbiFormat::Json | AbiFormat::JsonMin => "json",
        },
        AbiCompression::Zstd => "zst",
    }
}
