use camino::Utf8PathBuf;

use crate::extended::{BuildOptsExtended, EnvPairs};

fn is_wasm_build_skipped(build_skipped_when_env_is: &EnvPairs) -> bool {
    for (key, skip_value) in build_skipped_when_env_is.0.iter() {
        if let Ok(actual_value) = std::env::var(key) {
            if actual_value == *skip_value {
                return true;
            }
        }
    }
    false
}
/// Return value: [`Result::Ok`] is path to the wasm artifact obtained.
pub fn run(opts: BuildOptsExtended) -> eyre::Result<Utf8PathBuf> {
    let skip_build = is_wasm_build_skipped(&opts.build_skipped_when_env_is);

    let out_path = run_build::step(opts, skip_build)?;

    Ok(out_path)
}

mod run_build {
    use camino::Utf8PathBuf;
    use eyre::ContextCompat;

    use crate::extended::BuildOptsExtended;
    const EMPTY_WASM_STUB_FILENAME: &str = "empty_subcontract_stub.wasm";
    const EXPECT_OVERRIDE_TARGET_SET_MSG: &str = "[`BuildOpts::override_cargo_target_dir`] is expected to always be set in context of [`BuildOptsExtended`]";

    pub fn step(opts: BuildOptsExtended, skip_build: bool) -> eyre::Result<Utf8PathBuf> {
        if skip_build {
            let out_path = {
                let override_target = opts
                    .build_opts
                    .override_cargo_target_dir
                    .wrap_err(EXPECT_OVERRIDE_TARGET_SET_MSG)?;
                let override_target = Utf8PathBuf::from(override_target);

                override_target.join(EMPTY_WASM_STUB_FILENAME)
            };
            std::fs::write(&out_path, b"")?;
            Ok(out_path)
        } else {
            let out_path = crate::build_with_cli(opts.build_opts)?;
            Ok(out_path)
        }
    }
}
