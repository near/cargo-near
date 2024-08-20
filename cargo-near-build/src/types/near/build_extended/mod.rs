pub mod build_script;

use crate::types::near::build::Opts;

#[derive(Debug, Clone)]
pub struct OptsExtended<'a> {
    pub workdir: &'a str,
    /// vector of key-value pairs of temporary env overrides during build process
    pub env: Vec<(&'a str, &'a str)>,
    pub build_opts: Opts,
    pub build_script_opts: build_script::Opts<'a>,
}
