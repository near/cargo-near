use crate::BuildArtifact;
use rustc_version::Version;

/// `cargo::` prefix for build script outputs, that `cargo` recognizes
/// was implemented <https://github.com/rust-lang/cargo/pull/12201> since this version
const DEPRECATE_SINGLE_COLON_SINCE: Version = Version::new(1, 77, 0);

macro_rules! print_warn {
    ($version: ident, $($tokens: tt)*) => {
        let separator = if $version >= &DEPRECATE_SINGLE_COLON_SINCE {
            "::"
        } else {
            ":"
        };
        println!("cargo{}warning={}", separator, format!($($tokens)*))
    }
}

#[derive(Debug, Clone)]
pub struct BuildScriptOpts<'a> {
    /// environment variable name to export result `*.wasm` path to with [`cargo::rustc-env=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env)
    /// instruction
    pub result_env_key: Option<&'a str>,
    /// list of paths for [`cargo::rerun-if-changed=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
    /// instruction
    ///
    /// if relative, it's relative to path of crate, where build.rs is compiled
    pub rerun_if_changed_list: Vec<&'a str>,
    /// vector of key-value pairs of environment variable name and its value,
    /// when compilation should be skipped on a variable's value match;
    /// e.g.
    /// skipping emitting output `*.wasm` may be helpful when `PROFILE` is equal to `debug`
    /// for using  `rust-analyzer/flycheck`, `cargo check`, `bacon` and other dev-tools
    pub build_skipped_when_env_is: Vec<(&'a str, &'a str)>,
    /// path of stub file, where a placeholder empty `wasm` output is emitted to, when
    /// build is skipped due to match in [`Self::build_skipped_when_env_is`]
    ///
    /// if this path is relative, then the base is [`crate::BuildOptsExtended::workdir`]
    pub stub_path: Option<&'a str>,
    /// substitution export of [`CARGO_TARGET_DIR`](https://doc.rust-lang.org/cargo/reference/environment-variables.html),
    /// which is required to avoid deadlock <https://github.com/rust-lang/cargo/issues/8938>;
    /// should best be a subfolder of [`CARGO_TARGET_DIR`](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
    /// of crate being built to work normally in docker builds
    ///
    /// if this path is relative, then the base is [`crate::BuildOptsExtended::workdir`]
    pub distinct_target_dir: Option<&'a str>,
}

impl<'a> BuildScriptOpts<'a> {
    pub fn should_skip(&self, version: &Version) -> bool {
        let mut return_bool = false;
        for (env_key, value_to_skip) in self.build_skipped_when_env_is.iter() {
            if let Ok(actual_value) = std::env::var(env_key) {
                if actual_value == *value_to_skip {
                    return_bool = true;
                    print_warn!(
                        version,
                        "`{}` env set to `{}`, build was configured to skip on this value",
                        env_key,
                        actual_value
                    );
                }
            }
        }

        return_bool
    }
    pub fn create_empty_stub(&self) -> Result<BuildArtifact, Box<dyn std::error::Error>> {
        if self.stub_path.is_none() {
            return Err(
                "build must be skipped, but `BuildScriptOpts.stub_path` wasn't configured"
                    .to_string(),
            )?;
        }
        let stub_path = std::path::Path::new(self.stub_path.as_ref().unwrap());
        create_stub_file(stub_path)?;
        let stub_path = stub_path.canonicalize()?;

        let artifact = {
            let stub_path = camino::Utf8PathBuf::from_path_buf(stub_path)
                .map_err(|err| format!("`{}` isn't a valid UTF-8 path", err.to_string_lossy()))?;
            BuildArtifact {
                path: stub_path,
                fresh: true,
                from_docker: false,
                cargo_near_version_mismatch: None,
            }
        };
        Ok(artifact)
    }

    pub fn post_build(
        &self,
        skipped: bool,
        artifact: &BuildArtifact,
        workdir: &str,
        version: &Version,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let colon_separator = if version >= &DEPRECATE_SINGLE_COLON_SINCE {
            "::"
        } else {
            ":"
        };
        if let Some(ref version_mismatch) = artifact.cargo_near_version_mismatch {
            print_warn!(
                version,
                "WARNING: `cargo-near` version was coerced during build: {}",
                version_mismatch
            );
        }
        if let Some(ref result_env_key) = self.result_env_key {
            pretty_print(skipped, artifact, version)?;
            println!(
                "cargo{}rustc-env={}={}",
                colon_separator,
                result_env_key,
                artifact.path.clone().into_string()
            );
            print_warn!(
                version,
                "Path to result artifact of build in `{}` is exported to `{}`",
                workdir,
                result_env_key,
            );
        }
        for path in self.rerun_if_changed_list.iter() {
            println!("cargo{}rerun-if-changed={}", colon_separator, path);
        }
        Ok(())
    }
}

fn create_stub_file(out_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(out_path)?;
    Ok(())
}

fn pretty_print(
    skipped: bool,
    artifact: &BuildArtifact,
    version: &Version,
) -> Result<(), Box<dyn std::error::Error>> {
    if skipped {
        print_warn!(
            version,
            "Build empty artifact stub-file written to: `{}`",
            artifact.path.clone().into_string()
        );
        return Ok(());
    }
    let hash = artifact.compute_hash()?;

    print_warn!(version, "");
    print_warn!(version, "");
    print_warn!(
        version,
        "Build artifact path: {}",
        artifact.path.clone().into_string()
    );
    print_warn!(
        version,
        "Sub-build artifact SHA-256 checksum hex: {}",
        hash.to_hex_string()
    );
    print_warn!(
        version,
        "Sub-build artifact SHA-256 checksum bs58: {}",
        hash.to_base58_string()
    );
    print_warn!(version, "");
    print_warn!(version, "");
    Ok(())
}
