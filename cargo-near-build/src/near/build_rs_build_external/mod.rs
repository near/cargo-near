use camino::Utf8PathBuf;

use crate::extended::{BuildOptsExtended, EnvPairs};

fn is_wasm_build_skipped(build_skipped_when_env_is: &EnvPairs) -> bool {
    for (key, skip_value) in build_skipped_when_env_is.0.iter() {
        #[allow(clippy::collapsible_if)]
        if let Ok(actual_value) = std::env::var(key) {
            if actual_value == *skip_value {
                return true;
            }
        }
    }
    false
}
/// This is intended for use in `build.rs` of factories to build sub-contracts,
/// not for regular builds, where [`crate::build_with_cli`] is sufficient.
///
/// Return value: [`Result::Ok`] is path to the wasm artifact obtained.
pub fn run(opts: BuildOptsExtended) -> eyre::Result<Utf8PathBuf> {
    let skip_build = is_wasm_build_skipped(&opts.build_skipped_when_env_is);

    let out_path = run_build::step(opts.clone(), skip_build)?;

    post_build::step(
        skip_build,
        opts.rerun_if_changed_list,
        &out_path,
        opts.result_file_path_env_key,
    )?;

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

mod post_build {
    use camino::Utf8PathBuf;

    /// `cargo::` prefix for build script outputs, that `cargo` recognizes
    /// was implemented <https://github.com/rust-lang/cargo/pull/12201> since this version
    const DEPRECATE_SINGLE_COLON_SINCE: rustc_version::Version =
        rustc_version::Version::new(1, 77, 0);

    fn colon_separator(version: &rustc_version::Version) -> &str {
        if version >= &DEPRECATE_SINGLE_COLON_SINCE {
            "::"
        } else {
            ":"
        }
    }
    macro_rules! print_warn {
        ($version: expr, $($tokens: tt)*) => {
            let separator = colon_separator($version);
            println!("cargo{}warning={}", separator, format!($($tokens)*))
        }
    }

    fn print_result_env_var(
        result_env_var: String,
        out_path: &Utf8PathBuf,
        rust_version: &rustc_version::Version,
    ) {
        let colon_separator = colon_separator(rust_version);
        println!(
            "cargo{}rustc-env={}={}",
            colon_separator,
            result_env_var,
            out_path.as_str()
        );
        print_warn!(
            rust_version,
            "RESULT: path to wasm file of built or empty-stubbed sub-contract exported to `{}` env variable",
            result_env_var,
        );
    }
    pub fn step(
        skip_build: bool,
        rerun_if_changed_list: Vec<String>,
        out_path: &Utf8PathBuf,
        result_env_var: String,
    ) -> eyre::Result<()> {
        let rust_version = rustc_version::version()?;
        let colon_separator = colon_separator(&rust_version);

        for path in rerun_if_changed_list.iter() {
            println!("cargo{colon_separator}rerun-if-changed={path}");
        }

        if skip_build {
            print_warn!(
                &rust_version,
                "sub-contract empty stub is `{}`",
                out_path.as_str()
            );
        } else {
            print_warn!(
                &rust_version,
                "sub-contract wasm built out path is `{}`",
                out_path.as_str()
            );
        }

        if !skip_build {
            print_warn!(
                &rust_version,
                "subcontract sha256 is {}",
                crate::SHA256Checksum::new(out_path)?.to_base58_string()
            );
        }
        print_result_env_var(result_env_var, out_path, &rust_version);

        Ok(())
    }
}
