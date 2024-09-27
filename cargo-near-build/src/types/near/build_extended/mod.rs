use crate::extended::BuildScriptOpts;
use crate::BuildOpts;

pub mod build_script;

#[derive(Debug, Clone)]
pub struct OptsExtended<'a> {
    pub workdir: &'a str,
    pub build_opts: BuildOpts,
    pub build_script_opts: BuildScriptOpts<'a>,
}
