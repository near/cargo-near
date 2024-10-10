use crate::env_keys;

pub struct AbiPath {
    path: Option<camino::Utf8PathBuf>,
}

impl AbiPath {
    pub fn new(no_embed_abi: bool, min_abi_path: &Option<camino::Utf8PathBuf>) -> Self {
        if !no_embed_abi {
            Self {
                path: min_abi_path.clone(),
            }
        } else {
            Self { path: None }
        }
    }

    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        if let Some(ref path) = self.path {
            env.push((env_keys::CARGO_NEAR_ABI_PATH, path.as_str()));
        }
    }
}
