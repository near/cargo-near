use crate::{
    commands::build_command::{BUILD_RS_ABI_STEP_HINT_ENV_KEY, NEP330_CONTRACT_PATH_ENV_KEY},
    BuildArtifact, BuildOpts,
};

macro_rules! print_warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

pub struct OptsExtended<'a> {
    pub workdir: &'a str,
    /// the desired value of `contract_path` from `BuildInfo`
    /// <https://github.com/near/NEPs/blob/master/neps/nep-0330.md?plain=1#L155>
    pub metadata_contract_path: &'a str,
    pub build_opts: BuildOpts,
    /// substitution export of `CARGO_TARGET_DIR`,
    /// which is required to avoid deadlock <https://github.com/rust-lang/cargo/issues/8938>
    /// should be a subfolder of `CARGO_TARGET_DIR` of package being built to work normally in
    /// docker builds
    ///
    /// if this path is relative, then the base is `workdir` field
    pub distinct_target_dir: &'a str,
    /// skipping emitting output sub-build `*.wasm` may be helpful in `debug` profile, when
    /// interacting with `rust-analyzer/flycheck`,
    /// `cargo check`, `bacon` and other dev-tools, running `cargo test --workspace`, etc.
    pub skipped_profiles: Vec<&'a str>,
    /// path of stub file, where a placeholder empty `wasm` output is emitted to, when
    /// build is skipped
    pub stub_path: &'a str,
    /// list of paths for [`cargo:rerun-if-changed=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
    /// instruction
    pub rerun_if_changed_list: Vec<&'a str>,
}

pub fn build(
    args: OptsExtended,
    result_env_key: String,
) -> Result<BuildArtifact, Box<dyn std::error::Error>> {
    let (artifact, skipped) = skip_or_compile(&args, &result_env_key)?;
    print_warn!(
        "Path to result artifact of build in `{}` is exported to `{}`",
        &args.workdir,
        result_env_key,
    );
    Ok(artifact)
}

// TODO: replace `cargo:` -> `cargo::`, as the former is being deprecated since rust 1.77
// or handle both with `rustc_version`
fn skip_or_compile(
    args: &OptsExtended,
    result_env_key: &String,
) -> Result<(BuildArtifact, bool), Box<dyn std::error::Error>> {
    let result = if skip(args) {
        let stub_path = std::path::Path::new(&args.stub_path);
        create_stub_file(stub_path)?;

        let artifact = {
            let stub_path = camino::Utf8PathBuf::from_path_buf(stub_path.to_path_buf())
                .map_err(|err| format!("`{}` isn't a valid UTF-8 path", err.to_string_lossy()))?;
            BuildArtifact {
                path: stub_path,
                fresh: true,
                from_docker: false,
            }
        };
        let stub_path = stub_path
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();
        print_warn!("Sub-build empty artifact stub written to: `{}`", stub_path);
        println!("cargo:rustc-env={}={}", result_env_key, stub_path);
        (artifact, true)
    } else {
        let artifact = compile_near_artifact(args)?;
        pretty_print(&artifact)?;
        println!(
            "cargo:rustc-env={}={}",
            result_env_key,
            artifact.path.clone().into_string()
        );
        for path in args.rerun_if_changed_list.iter() {
            println!("cargo:rerun-if-changed={}", path);
        }
        (artifact, false)
    };
    Ok(result)
}

fn skip(args: &OptsExtended) -> bool {
    let profile = std::env::var("PROFILE").unwrap_or("unknown".to_string());
    print_warn!("`PROFILE` env set to `{}`", profile);

    if args.skipped_profiles.contains(&profile.as_str()) {
        print_warn!(
            "No need to build factory's product contract during `{}` profile build",
            profile
        );
        return true;
    }
    if std::env::var(BUILD_RS_ABI_STEP_HINT_ENV_KEY).is_ok() {
        print_warn!("No need to build factory's product contract during ABI generation step");
        return true;
    }
    false
}

/// `CARGO_NEAR_BUILD_COMMAND` and `CARGO_NEAR_CONTRACT_PATH`
/// exports ensure, that contract, deployed from factory, produces the same metadata
/// as one, deployed by `cargo near deploy` from `product-donation` subfolder,
/// (in the context of docker builds)
///
/// `CARGO_TARGET_DIR` export is needed to avoid attempt to acquire same `target/<profile-path>/.cargo-lock`
/// as the `cargo` process, which is running the build-script
fn compile_near_artifact(args: &OptsExtended) -> Result<BuildArtifact, Box<dyn std::error::Error>> {
    let _tmp_workdir = tmp_env::set_current_dir(args.workdir)?;

    let _tmp_contract_path_env =
        tmp_env::set_var(NEP330_CONTRACT_PATH_ENV_KEY, args.metadata_contract_path);

    let _tmp_cargo_target_env = tmp_env::set_var("CARGO_TARGET_DIR", args.distinct_target_dir);
    let artifact = crate::build(args.build_opts.clone())?;

    Ok(artifact)
}

fn create_stub_file(out_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(out_path)?;
    Ok(())
}

fn pretty_print(artifact: &BuildArtifact) -> Result<(), Box<dyn std::error::Error>> {
    let hash = artifact.compute_hash()?;

    print_warn!("");
    print_warn!("");
    print_warn!(
        "Sub-build artifact path: {}",
        artifact.path.clone().into_string()
    );
    print_warn!("Sub-build artifact SHA-256 checksum hex: {}", hash.hex);
    print_warn!("Sub-build artifact SHA-256 checksum bs58: {}", hash.base58);
    print_warn!("");
    print_warn!("");
    Ok(())
}
