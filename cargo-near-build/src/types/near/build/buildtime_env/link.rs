use crate::{env_keys, types::cargo::metadata::CrateMetadata};

pub struct Nep330Link {
    link: Option<String>,
}

impl Nep330Link {
    pub fn new(crate_metadata: &CrateMetadata) -> Self {
        Self {
            link: crate_metadata.root_package.repository.clone(),
        }
    }

    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        // this will be set in docker builds (externally to current process), having more info about git commit
        if std::env::var(env_keys::nep330::LINK).is_err() {
            if let Some(ref link) = self.link {
                env.push((env_keys::nep330::LINK, link.as_str()));
            }
        }
    }
}
