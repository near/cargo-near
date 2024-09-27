mod build_script;
mod tmp_change_cwd;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::build_extended::OptsExtended;
use crate::BuildOpts;
use rustc_version::Version;

use crate::extended::BuildScriptOpts;

pub fn run(args: OptsExtended) -> Result<CompilationArtifact, Box<dyn std::error::Error>> {
    let actual_version = rustc_version::version()?;
    let OptsExtended {
        workdir,
        build_opts,
        build_script_opts,
    } = args;
    let (artifact, skipped) =
        skip_or_compile(workdir, build_opts, &build_script_opts, &actual_version)?;

    build_script_opts.post_build(skipped, &artifact, workdir, &actual_version)?;
    Ok(artifact)
}

pub(crate) fn skip_or_compile(
    workdir: &'_ str,
    build_opts: BuildOpts,
    build_script_opts: &BuildScriptOpts<'_>,
    version: &Version,
) -> Result<(CompilationArtifact, bool), Box<dyn std::error::Error>> {
    let _tmp_workdir = tmp_change_cwd::set_current_dir(workdir)?;
    let result = if build_script_opts.should_skip(version) {
        let artifact = build_script_opts.create_empty_stub()?;
        (artifact, true)
    } else {
        let artifact = compile_near_artifact(build_opts, build_script_opts)?;
        (artifact, false)
    };
    Ok(result)
}

/// `CARGO_TARGET_DIR` export is needed to avoid attempt to acquire same `target/<profile-path>/.cargo-lock`
/// as the `cargo` process, which is running the build-script
pub(crate) fn compile_near_artifact(
    mut build_opts: BuildOpts,
    build_script_opts: &BuildScriptOpts<'_>,
) -> Result<CompilationArtifact, Box<dyn std::error::Error>> {
    if let Some(distinct_target_dir) = build_script_opts.distinct_target_dir {
        build_opts.mute_env = {
            let mut mute_env = build_opts.mute_env;
            mute_env.push((
                "CARGO_TARGET_DIR".to_string(),
                distinct_target_dir.to_string(),
            ));
            mute_env
        };
    }
    let artifact = crate::build(build_opts.clone())?;

    Ok(artifact)
}
