use crate::env_keys;

// TODO #F: uncomment for `build_external_extended` method
#[allow(unused)]
pub enum CargoTargetDir {
    Set(String),
    NoOp,
    #[allow(unused)]
    UnsetExternal,
}

// TODO #F: uncomment for `build_external_extended` method
#[allow(unused)]
impl CargoTargetDir {
    pub fn new(path: Option<String>) -> Self {
        match path {
            Some(path) => Self::Set(path),
            None => Self::NoOp,
        }
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        if let Self::Set(path) = self {
            env.push((env_keys::CARGO_TARGET_DIR, path.as_str()))
        }
    }

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
