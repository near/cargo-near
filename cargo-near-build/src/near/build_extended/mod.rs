macro_rules! print_warn {
    ($version: expr, $($tokens: tt)*) => {
        let separator = if $version >= &DEPRECATE_SINGLE_COLON_SINCE {
            "::"
        } else {
            ":"
        };
        println!("cargo{}warning={}", separator, format!($($tokens)*))
    }
}

/// `cargo::` prefix for build script outputs, that `cargo` recognizes
/// was implemented <https://github.com/rust-lang/cargo/pull/12201> since this version
const DEPRECATE_SINGLE_COLON_SINCE: Version = Version::new(1, 77, 0);

mod build_script;

use crate::types::near::build::buildtime_env;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::build_extended::OptsExtended;
use crate::BuildOpts;
use rustc_version::Version;

use crate::extended::BuildScriptOpts;

use super::build::get_crate_metadata;

pub fn run(args: OptsExtended) -> Result<CompilationArtifact, Box<dyn std::error::Error>> {
    let actual_version = rustc_version::version()?;
    print_warn!(
        &actual_version,
        "build script of `{}` is happening in workdir: {:?}",
        std::env::var("CARGO_PKG_NAME").unwrap_or("unset CARGO_PKG_NAME".into()),
        std::env::current_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or("ERR GET PWD".into())
    );
    let OptsExtended {
        build_opts,
        build_script_opts,
    } = args;
    let (artifact, skipped) = skip_or_compile(build_opts, &build_script_opts, &actual_version)?;

    build_script_opts.post_build(skipped, &artifact, &actual_version)?;
    Ok(artifact)
}

pub(crate) fn skip_or_compile(
    mut build_opts: BuildOpts,
    build_script_opts: &BuildScriptOpts,
    version: &Version,
) -> Result<(CompilationArtifact, bool), Box<dyn std::error::Error>> {
    let result = if build_script_opts.should_skip(version) {
        let artifact = build_script_opts.create_empty_stub()?;
        (artifact, true)
    } else {
        if build_opts.override_cargo_target_dir.is_some() {
            let metadata =
                get_crate_metadata(&build_opts, &buildtime_env::CargoTargetDir::UnsetExternal)?;
            let output_paths = metadata.get_legacy_cargo_near_output_path(None)?;
            build_opts.override_output_wasm_path = Some(output_paths.get_wasm_file().to_string());
        }
        let artifact = crate::build(build_opts)?;
        (artifact, false)
    };
    Ok(result)
}
