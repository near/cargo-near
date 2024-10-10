mod build_script;
mod tmp_change_cwd;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::build_extended::OptsExtended;
use crate::{BuildImplicitEnvOpts, BuildOpts};
use rustc_version::Version;

use crate::extended::BuildScriptOpts;

/// only single-threaded build-scripts are supported
///
/// this function cannot be run concurrently with itself, as it changes working dir
/// of process, executing it.
/// changing current working dir concurrently from different threads
/// may entail incorrect results
pub fn run(args: OptsExtended) -> Result<CompilationArtifact, Box<dyn std::error::Error>> {
    let actual_version = rustc_version::version()?;
    let OptsExtended {
        workdir,
        build_opts,
        build_script_opts,
        build_implicit_env_opts,
    } = args;
    let (artifact, skipped) = skip_or_compile(
        &workdir,
        build_opts,
        build_implicit_env_opts,
        &build_script_opts,
        &actual_version,
    )?;

    build_script_opts.post_build(skipped, &artifact, workdir, &actual_version)?;
    Ok(artifact)
}

pub(crate) fn skip_or_compile(
    workdir: &'_ str,
    build_opts: BuildOpts,
    build_implicit_env_opts: BuildImplicitEnvOpts,
    build_script_opts: &BuildScriptOpts,
    version: &Version,
) -> Result<(CompilationArtifact, bool), Box<dyn std::error::Error>> {
    let _tmp_workdir = tmp_change_cwd::set_current_dir(workdir)?;
    let result = if build_script_opts.should_skip(version) {
        let artifact = build_script_opts.create_empty_stub()?;
        (artifact, true)
    } else {
        let artifact = crate::build(build_opts, Some(build_implicit_env_opts))?;
        (artifact, false)
    };
    Ok(result)
}
