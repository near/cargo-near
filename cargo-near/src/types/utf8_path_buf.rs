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
pub struct Utf8PathBuf(pub camino::Utf8PathBuf);

impl std::fmt::Display for Utf8PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl interactive_clap::ToCli for Utf8PathBuf {
    type CliVariant = Utf8PathBuf;
}
