use std::env;
use std::io::IsTerminal;

#[derive(Debug, Clone)]
pub enum ColorPreference {
    Auto,
    Always,
    Never,
}

impl std::fmt::Display for ColorPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
        }
    }
}

fn default_mode() -> ColorPreference {
    match env::var("NO_COLOR") {
        Ok(v) if v != "0" => ColorPreference::Never,
        _ => {
            if std::io::stderr().is_terminal() {
                ColorPreference::Always
            } else {
                ColorPreference::Never
            }
        }
    }
}

impl ColorPreference {
    pub fn as_str(&self) -> &str {
        match self {
            ColorPreference::Auto => "auto",
            ColorPreference::Always => "always",
            ColorPreference::Never => "never",
        }
    }

    pub fn apply(&self) {
        match self {
            ColorPreference::Auto => {
                default_mode().apply();
            }
            ColorPreference::Always => colored::control::set_override(true),
            ColorPreference::Never => colored::control::set_override(false),
        }
    }
}
