#[cfg(feature = "build_internal")]
use crate::types::near::build::buildtime_env::BuilderAbiVersions;

pub type Version = String;

#[derive(Debug)]
#[allow(unused)]
pub(crate) enum VersionInfo {
    EnvMismatch {
        environment: Version,
        current_process: Version,
    },
    CurrentProcess(Version),
    UnknownFromDocker,
}

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnvMismatch {
                environment,
                current_process,
            } => {
                write!(
                    f,
                    "builder version `{}` -> builder environment version `{}`",
                    current_process, environment
                )
            }
            Self::CurrentProcess { .. } => write!(f, "no `cargo-near` version mismatch in nested builds detected",),
            Self::UnknownFromDocker => write!(f, "it's unknown if `cargo-near` version mismatch has occurred in docker build environment",),
        }
    }
}

#[cfg(feature = "build_internal")]
impl VersionInfo {
    pub fn result_builder_version(&self) -> eyre::Result<Version> {
        match self {
            Self::EnvMismatch { environment, .. } => Ok(environment.clone()),
            Self::CurrentProcess(current_process) => Ok(current_process.clone()),
            Self::UnknownFromDocker => Err(eyre::eyre!(
                "info about Version mismatch is unknown \
                for docker build",
            )),
        }
    }

    fn current_builder_version() -> Version {
        format!("{} {}", "cargo-near-build", env!("CARGO_PKG_VERSION"))
    }

    fn current_near_abi_schema_version() -> Version {
        near_abi::SCHEMA_VERSION.into()
    }
    pub fn compute_env_variables(&self) -> eyre::Result<BuilderAbiVersions> {
        let builder_version = self.result_builder_version()?;
        let v = BuilderAbiVersions::new(builder_version, Self::current_near_abi_schema_version());
        Ok(v)
    }

    pub fn get_coerced_builder_version() -> eyre::Result<VersionInfo> {
        Self::check_near_abi_schema_mismatch()?;

        let current_process_version = Self::current_builder_version();
        let result = match std::env::var(BuilderAbiVersions::builder_version_env_key()) {
            Err(_err) => VersionInfo::CurrentProcess(current_process_version),
            Ok(env_version) => match env_version == current_process_version {
                true => VersionInfo::CurrentProcess(current_process_version),
                // coercing to env_version on mismatch
                false => VersionInfo::EnvMismatch {
                    environment: env_version,
                    current_process: current_process_version,
                },
            },
        };
        Ok(result)
    }

    fn check_near_abi_schema_mismatch() -> eyre::Result<()> {
        match std::env::var(BuilderAbiVersions::abi_schema_version_env_key()) {
            Ok(env_near_abi_schema_version) => {
                if env_near_abi_schema_version != Self::current_near_abi_schema_version() {
                    Err(eyre::eyre!(
                        "current process NEAR_ABI_SCHEMA_VERSION mismatch with env value: {} vs {}",
                        Self::current_near_abi_schema_version(),
                        env_near_abi_schema_version,
                    ))
                } else {
                    Ok(())
                }
            }
            Err(_err) => Ok(()),
        }
    }
}
