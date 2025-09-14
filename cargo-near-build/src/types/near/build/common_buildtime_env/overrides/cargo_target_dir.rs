use crate::env_keys;

pub enum CargoTargetDir {
    #[allow(unused)]
    Set(String),
    #[allow(unused)]
    NoOp,
    #[allow(unused)]
    UnsetExternal,
}

impl CargoTargetDir {
    #[cfg(feature = "build_internal")]
    pub fn new(path: Option<String>) -> Self {
        match path {
            Some(path) => Self::Set(path),
            None => Self::NoOp,
        }
    }
    #[cfg(feature = "build_internal")]
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        if let Self::Set(path) = self {
            env.push((env_keys::CARGO_TARGET_DIR, path.as_str()))
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_std_command(&self, cmd: &mut std::process::Command) {
        match self {
            Self::NoOp => {}
            Self::Set(path) => {
                cmd.env(env_keys::CARGO_TARGET_DIR, path.as_str());
            }
            Self::UnsetExternal => {
                cmd.env_remove(env_keys::CARGO_TARGET_DIR);
            }
        }
    }
}
