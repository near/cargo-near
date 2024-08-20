use crate::{BuildOpts, BuildScriptOpts};

pub mod build_script;

#[derive(Debug, Clone)]
pub struct OptsExtended<'a> {
    pub workdir: &'a str,
    /// vector of key-value pairs of temporary env overrides during build process
    pub env: Vec<(&'a str, &'a str)>,
    pub build_opts: BuildOpts,
    pub build_script_opts: BuildScriptOpts<'a>,
}
