//! TODO: replace `cargo:` -> `cargo::`, as the former is being deprecated since rust 1.77
//! or handle both with `rustc_version`
use crate::BuildArtifact;

macro_rules! print_warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

#[derive(Debug, Clone)]
pub struct BuildScriptOpts<'a> {
    /// key to export result path to with [`cargo:rustc-env=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env)
    /// instruction
    pub result_env_key: Option<&'a str>,
    /// list of paths for [`cargo:rerun-if-changed=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
    /// instruction
    ///
    /// if relative, it's relative to path of crate, where build.rs is compiled
    pub rerun_if_changed_list: Vec<&'a str>,
    /// vector of key-value pairs of env variable name and its value,
    /// when compiling should be skipped on env variable's value match;
    /// e.g.
    /// skipping emitting output sub-build `*.wasm` may be helpful when `PROFILE` is equal to `debug`
    /// for using  `rust-analyzer/flycheck`, `cargo check`, `bacon` and other dev-tools
    pub build_skipped_when_env_is: Vec<(&'a str, &'a str)>,
    /// path of stub file, where a placeholder empty `wasm` output is emitted to, when
    /// build is skipped due to match in `skipped_env_values`
    ///
    /// if this path is relative, then the base is `workdir` field of `OptsExtended`
    pub stub_path: Option<&'a str>,
    /// substitution export of `CARGO_TARGET_DIR`,
    /// which is required to avoid deadlock <https://github.com/rust-lang/cargo/issues/8938>
    /// should be a subfolder of `CARGO_TARGET_DIR` of package being built to work normally in
    /// docker builds
    ///
    /// if this path is relative, then the base is `workdir` field of `OptsExtended`
    pub distinct_target_dir: Option<&'a str>,
}

impl<'a> BuildScriptOpts<'a> {
    pub fn should_skip(&self) -> bool {
        let mut return_bool = false;
        for (env_key, value_to_skip) in self.build_skipped_when_env_is.iter() {
            if let Ok(actual_value) = std::env::var(env_key) {
                if actual_value == *value_to_skip {
                    return_bool = true;
                    print_warn!(
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
            }
        };
        Ok(artifact)
    }

    pub fn post_build(
        &self,
        skipped: bool,
        artifact: &BuildArtifact,
        workdir: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref result_env_key) = self.result_env_key {
            pretty_print(skipped, artifact)?;
            println!(
                "cargo:rustc-env={}={}",
                result_env_key,
                artifact.path.clone().into_string()
            );
            print_warn!(
                "Path to result artifact of build in `{}` is exported to `{}`",
                workdir,
                result_env_key,
            );
        }
        for path in self.rerun_if_changed_list.iter() {
            println!("cargo:rerun-if-changed={}", path);
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

fn pretty_print(skipped: bool, artifact: &BuildArtifact) -> Result<(), Box<dyn std::error::Error>> {
    if skipped {
        print_warn!(
            "Build empty artifact stub-file written to: `{}`",
            artifact.path.clone().into_string()
        );
        return Ok(());
    }
    let hash = artifact.compute_hash()?;

    print_warn!("");
    print_warn!("");
    print_warn!(
        "Build artifact path: {}",
        artifact.path.clone().into_string()
    );
    print_warn!("Sub-build artifact SHA-256 checksum hex: {}", hash.hex);
    print_warn!("Sub-build artifact SHA-256 checksum bs58: {}", hash.base58);
    print_warn!("");
    print_warn!("");
    Ok(())
}
