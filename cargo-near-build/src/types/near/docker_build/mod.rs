use crate::{BuildContext, BuildOpts};

pub mod crate_in_repo;
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
