use bon::bon;
use camino::Utf8PathBuf;
use eyre::{Context, ContextCompat};

use crate::types::cargo::metadata::CrateMetadata;
use crate::types::{cargo::manifest_path::ManifestPath, near::build::common_buildtime_env};

#[derive(Debug, Clone)]
pub struct BuildOptsExtended {
    pub build_opts: crate::BuildOpts,
    pub build_skipped_when_env_is: EnvPairs,
    pub rerun_if_changed_list: Vec<String>,
    pub result_file_path_env_key: String,
}

#[bon]
impl BuildOptsExtended {
    #[builder(finish_fn = prepare)]
    pub fn new(
        mut build_opts: crate::BuildOpts,
        /// vector of (`environment_variable_key`, `skip_value`),
        ///
        /// which are config on when to
        /// skip the build of subcontract's wasm
        /// (when value of `environment_variable_key` is equal to `skip_value`)
        ///
        /// Default value:
        /// ```rust
        /// # let value: EnvPairs =
        /// vec![
        ///        // shorter build for `cargo check`
        ///        (crate::env_keys::RUST_PROFILE, "debug"),
        ///        // skip build of subcontract when ABI is being generated for current contract
        ///        (crate::env_keys::BUILD_RS_ABI_STEP_HINT, "true"),
        ///    ]
        ///    .into()
        /// # ;
        /// ```
        #[builder(default, into)]
        mut build_skipped_when_env_is: EnvPairs,
        #[builder(default)] mut rerun_if_changed_list: Vec<String>,
        #[builder(into)] result_file_path_env_key: String,
        #[builder(default)] passed_env: Vec<String>,
    ) -> eyre::Result<Self> {
        if let None = build_opts.override_cargo_target_dir {
            build_opts.override_cargo_target_dir = Some(override_cargo_target_dir()?.into_string());
        }
        let workdir = ManifestPath::get_manifest_workdir(build_opts.manifest_path.clone())?;

        if let None = build_opts.override_nep330_contract_path {
            build_opts.override_nep330_contract_path = override_nep330_contract_path(&workdir)?;
        }

        if let None = build_opts.override_nep330_output_wasm_path {
            build_opts.override_nep330_output_wasm_path =
                Some(override_nep330_output_wasm_path(&build_opts)?);
        }

        let passed_env_entries = passed_env
            .into_iter()
            .filter_map(|key| std::env::var(&key).ok().map(|value| (key, value)));
        build_opts.env.extend(passed_env_entries);

        if build_skipped_when_env_is.0.is_empty() {
            build_skipped_when_env_is = vec![
                // shorter build for `cargo check`
                (crate::env_keys::RUST_PROFILE, "debug"),
                (crate::env_keys::BUILD_RS_ABI_STEP_HINT, "true"),
            ]
            .into();
        }

        if !rerun_if_changed_list.contains(&workdir.to_string()) {
            rerun_if_changed_list.push(workdir.to_string());
        }

        Ok(Self {
            build_opts,
            build_skipped_when_env_is,
            rerun_if_changed_list,
            result_file_path_env_key,
        })
    }
}

/// we don't have to honeslty compute a canonicalized relative path by appending
/// relative `build_opts.manifest` path to relative [`crate::env_keys::nep330::CONTRACT_PATH`]
/// and then bringing it to canonicalized form;
///
/// for all we know midterm [`crate::env_keys::nep330::CONTRACT_PATH`] is only
/// expected to be relevant in nep330 conformant build environments, and all of those have
/// repository roots mounted to [`crate::env_keys::nep330::NEP330_REPO_MOUNT`]
fn override_nep330_contract_path(workdir: &Utf8PathBuf) -> eyre::Result<Option<String>> {
    if let Ok(_contract_path) = std::env::var(crate::env_keys::nep330::CONTRACT_PATH) {
        if workdir.starts_with(crate::env_keys::nep330::NEP330_REPO_MOUNT) {
            let workdir_pathdiff = pathdiff::diff_utf8_paths(
                &workdir,
                Utf8PathBuf::from(crate::env_keys::nep330::NEP330_REPO_MOUNT.to_string()),
            )
            .wrap_err(format!(
                "cannot compute workdir `{}` relative path to base `{}`",
                workdir,
                crate::env_keys::nep330::NEP330_REPO_MOUNT,
            ))?;

            if !workdir_pathdiff.is_relative() {
                return Err(eyre::eyre!(
                    "workdir `{}` in `{}` isn't relative : {:?}",
                    workdir,
                    crate::env_keys::nep330::NEP330_REPO_MOUNT,
                    workdir_pathdiff.as_str()
                ));
            }

            // there's no need to additionally transform `workdir_pathdiff` into a valid
            // [unix_path::PathBuf], as the [`crate::env_keys::nep330::CONTRACT_PATH`] override
            // is only relevant inside of docker linux containers
            return Ok(Some(workdir_pathdiff.to_string()));
        }
    }
    Ok(None)
}
/// this is equal to wasm result path of `cargo near build non-reproducible-wasm`
/// for target sub-contract, when [`crate::env_keys::CARGO_TARGET_DIR`] is not set to any value,
/// which is the case when the sub-contract is being built as a stand-alone primary contract
/// outside of build.rs context
fn override_nep330_output_wasm_path(build_opts: &crate::BuildOpts) -> eyre::Result<String> {
    let metadata_with_no_target_override = CrateMetadata::get_with_build_opts(
        build_opts,
        &common_buildtime_env::CargoTargetDir::UnsetExternal,
    )?;
    let output_paths = metadata_with_no_target_override.get_legacy_cargo_near_output_path(None)?;
    Ok(output_paths.get_wasm_file().to_string())
}

fn override_cargo_target_dir() -> eyre::Result<Utf8PathBuf> {
    let out_dir_env = std::env::var("OUT_DIR")
        .wrap_err("unset OUT_DIR, which is expected to be always set for build.rs")?;
    let out_dir = Utf8PathBuf::from(out_dir_env);

    let dir = out_dir.join(format!("target-{}-for-{}", "product", "factory"));

    std::fs::create_dir_all(&dir).wrap_err(format!("couldn't create dir `{}`", dir))?;
    Ok(dir)
}

/// utility type which can be initialized with vector of 2-element tuples of literal strings,
/// by using [core::convert::Into]
/// like so: `vec![("key1", "value1"), ("key2", "value2")].into()`
#[derive(Default, Debug, Clone)]
pub struct EnvPairs(pub Vec<(String, String)>);

impl From<Vec<(&str, &str)>> for EnvPairs {
    fn from(value: Vec<(&str, &str)>) -> Self {
        let vector = value
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        Self(vector)
    }
}
