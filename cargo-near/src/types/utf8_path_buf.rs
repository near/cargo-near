use color_eyre::eyre::Context;

#[derive(
    Debug,
    Default,
    Clone,
    derive_more::AsRef,
    derive_more::From,
    derive_more::Into,
    derive_more::FromStr,
)]
#[as_ref(forward)]
pub struct Utf8PathBufInner(pub camino::Utf8PathBuf);

impl std::fmt::Display for Utf8PathBufInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl interactive_clap::ToCli for Utf8PathBufInner {
    type CliVariant = Utf8PathBufInner;
}

impl Utf8PathBufInner {
    pub fn read_bytes(&self) -> color_eyre::Result<Vec<u8>> {
        std::fs::read(self.0.clone().into_std_path_buf())
            .wrap_err_with(|| format!("Error reading data from file: {:?}", self.0))
    }
}
