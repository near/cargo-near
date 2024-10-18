use crate::extended::BuildScriptOpts;
use crate::BuildOpts;

pub mod build_script;

#[derive(Debug, Clone, bon::Builder)]
pub struct OptsExtended {
    pub build_opts: BuildOpts,
    pub build_script_opts: BuildScriptOpts,
}
