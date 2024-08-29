use crate::env_keys;

pub type Version = String;

#[derive(Debug)]
pub enum VersionMismatch {
    Some {
        environment: Version,
        current_process: Version,
    },
    None,
    UnknownFromDocker,
}

impl std::fmt::Display for VersionMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Some {
                environment,
                current_process,
            } => {
                write!(
                    f,
                    "builder version `{}` -> builder environment version `{}`",
                    current_process, environment
                )
            }
            Self::None => write!(f, "no `cargo-near` version mismatch in nested builds detected",),
            Self::UnknownFromDocker => write!(f, "it's unknown if `cargo-near` version mismatch has occurred in docker build environment",),
        }
    }
}

impl VersionMismatch {
    fn current_version() -> Version {
        format!("{} {}", "cargo-near-build", env!("CARGO_PKG_VERSION"))
    }

    pub(crate) fn export_builder_and_near_abi_versions() {
        if std::env::var(env_keys::CARGO_NEAR_VERSION).is_err() {
            std::env::set_var(env_keys::CARGO_NEAR_VERSION, Self::current_version());
        }
        if std::env::var(env_keys::CARGO_NEAR_ABI_SCHEMA_VERSION).is_err() {
            std::env::set_var(
                env_keys::CARGO_NEAR_ABI_SCHEMA_VERSION,
                near_abi::SCHEMA_VERSION,
            );
        }
    }

    pub(crate) fn get_coerced_builder_version() -> eyre::Result<(Version, VersionMismatch)> {
        match std::env::var(env_keys::CARGO_NEAR_ABI_SCHEMA_VERSION) {
            Ok(env_near_abi_schema_version) => {
                if env_near_abi_schema_version != near_abi::SCHEMA_VERSION {
                    return Err(eyre::eyre!(
                        "current process NEAR_ABI_SCHEMA_VERSION mismatch with env value: {} vs {}",
                        near_abi::SCHEMA_VERSION,
                        env_near_abi_schema_version,
                    ));
                }
            }
            Err(_err) => {}
        }
        let current_version = Self::current_version();

        let result = match std::env::var(env_keys::CARGO_NEAR_VERSION) {
            Err(_err) => (current_version.to_string(), VersionMismatch::None),
            Ok(env_version) => match env_version == current_version {
                true => (current_version.to_string(), VersionMismatch::None),
                // coercing to env_version on mismatch
                false => (
                    env_version.clone(),
                    VersionMismatch::Some {
                        environment: env_version,
                        current_process: current_version.to_string(),
                    },
                ),
            },
        };
        Ok(result)
    }
}
