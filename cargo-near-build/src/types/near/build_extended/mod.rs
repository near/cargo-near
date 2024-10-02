use crate::extended::BuildScriptOpts;
use crate::{BuildImplicitEnvOpts, BuildOpts};

pub mod build_script;

#[derive(Debug, Clone)]
pub struct OptsExtended {
    pub workdir: String,
    pub build_opts: BuildOpts,
    pub build_implicit_env_opts: BuildImplicitEnvOpts,
    pub build_script_opts: BuildScriptOpts,
}
