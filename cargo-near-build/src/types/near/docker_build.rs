use crate::{BuildContext, BuildOpts};

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
