use crate::env_keys;

pub struct BuilderAbiVersions {
    builder_version: String,
    near_abi_schema_version: String,
}

impl BuilderAbiVersions {
    pub fn new(builder_version: String, near_abi_schema_version: String) -> Self {
        Self {
            builder_version,
            near_abi_schema_version,
        }
    }
    pub fn builder_version_env_key() -> &'static str {
        env_keys::CARGO_NEAR_VERSION
    }
    pub fn abi_schema_version_env_key() -> &'static str {
        env_keys::CARGO_NEAR_ABI_SCHEMA_VERSION
    }

    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push((
            Self::builder_version_env_key(),
            self.builder_version.as_str(),
        ));
        env.push((
            Self::abi_schema_version_env_key(),
            self.near_abi_schema_version.as_str(),
        ));
    }
}
