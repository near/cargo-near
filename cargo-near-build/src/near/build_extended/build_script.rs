use std::marker::PhantomData;

use crate::types::near::build::output::version_mismatch::VersionMismatch;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::build_extended::build_script::Opts;
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

impl<'a> Opts<'a> {
    pub(crate) fn should_skip(&self, version: &Version) -> bool {
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
    pub(crate) fn create_empty_stub(
        &self,
    ) -> Result<CompilationArtifact, Box<dyn std::error::Error>> {
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
            CompilationArtifact {
                path: stub_path,
                fresh: true,
                from_docker: false,
                builder_version_mismatch: VersionMismatch::None,
                artifact_type: PhantomData,
            }
        };
        Ok(artifact)
    }

    pub(crate) fn post_build(
        &self,
        skipped: bool,
        artifact: &CompilationArtifact,
        workdir: &str,
        version: &Version,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let colon_separator = if version >= &DEPRECATE_SINGLE_COLON_SINCE {
            "::"
        } else {
            ":"
        };
        if let ref version_mismatch @ VersionMismatch::Some { .. } =
            artifact.builder_version_mismatch
        {
            print_warn!(
                version,
                "INFO: `cargo-near` version was coerced during build: {}.",
                version_mismatch
            );
            print_warn!(version, "`cargo-near` crate version (used in `build.rs`) did not match `cargo-near` build environment.");
            print_warn!(version, "You may consider to optionally make 2 following versions match exactly, if they're too far away:");
            print_warn!(
                version,
                "1. `cargo-near` CLI version being run in docker container, OR version of `cargo-near` CLI on host for a NO-Docker build."
            );
            print_warn!(
                version,
                "2. `cargo-near` version in `[build-dependencies]` in Cargo.toml."
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
    artifact: &CompilationArtifact,
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
