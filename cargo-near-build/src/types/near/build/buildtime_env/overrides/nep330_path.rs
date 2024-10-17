use crate::env_keys;

pub struct Nep330ContractPath {
    path: Option<String>,
}

impl Nep330ContractPath {
    pub fn maybe_new(path: Option<String>) -> Self {
        Self { path }
    }

    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        if let Some(ref path) = self.path {
            env.push((env_keys::nep330::CONTRACT_PATH, path.as_str()));
        }
    }
}
