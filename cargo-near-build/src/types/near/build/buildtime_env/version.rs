use crate::{env_keys, types::cargo::metadata::CrateMetadata};

pub struct Nep330Version {
    version: String,
}

impl Nep330Version {
    pub fn new(crate_metadata: &CrateMetadata) -> Self {
        Self {
            version: crate_metadata.root_package.version.to_string(),
        }
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push((env_keys::nep330::VERSION, self.version.as_str()));
    }
}
