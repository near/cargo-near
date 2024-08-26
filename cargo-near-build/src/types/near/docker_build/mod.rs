use crate::{BuildContext, BuildOpts};

pub mod cloned_repo;
pub mod container_paths;
pub mod crate_in_repo;
pub mod env_vars;
pub mod metadata;

#[derive(Default, Debug, Clone)]
pub struct Opts {
    pub build_opts: BuildOpts,
    pub context: BuildContext,
}

impl Default for BuildContext {
    fn default() -> Self {
        Self::Build
    }
}

pub const WARN_BECOMES_ERR: &str =
    "This WARNING becomes a hard ERROR when deploying contract with docker.";
