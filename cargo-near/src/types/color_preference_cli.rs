use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, EnumDiscriminants, Clone, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ColorPreferenceCli {
    Auto,
    Always,
    Never,
}

impl interactive_clap::ToCli for ColorPreferenceCli {
    type CliVariant = ColorPreferenceCli;
}

impl From<ColorPreferenceCli> for cargo_near_build::ColorPreference {
    fn from(value: ColorPreferenceCli) -> Self {
        match value {
            ColorPreferenceCli::Auto => Self::Auto,
            ColorPreferenceCli::Always => Self::Always,
            ColorPreferenceCli::Never => Self::Never,
        }
    }
}

impl std::fmt::Display for ColorPreferenceCli {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
        }
    }
}
