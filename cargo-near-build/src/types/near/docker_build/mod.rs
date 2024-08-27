use crate::BuildOpts;

use super::build::input::BuildContext;

pub mod cloned_repo;
pub mod crate_in_repo;
pub mod metadata;

pub mod subprocess;

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
