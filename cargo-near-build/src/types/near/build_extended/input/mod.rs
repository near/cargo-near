use bon::bon;
use camino::Utf8PathBuf;
use eyre::Context;

use crate::types::cargo::manifest_path::ManifestPath;

#[derive(Debug, Clone)]
pub struct BuildOptsExtended {
    pub build_opts: crate::BuildOpts,
    pub build_skipped_when_env_is: EnvPairs,
    pub rerun_if_changed_list: Vec<String>,
    pub result_file_path_env_key: String,
}

#[bon]
impl BuildOptsExtended {
    #[builder]
    pub fn new(
        mut build_opts: crate::BuildOpts,
        #[builder(default, into)] mut build_skipped_when_env_is: EnvPairs,
        #[builder(default)] mut rerun_if_changed_list: Vec<String>,
        #[builder(into)] result_file_path_env_key: String,
    ) -> eyre::Result<Self> {
        if let None = build_opts.override_cargo_target_dir {
            build_opts.override_cargo_target_dir = Some(override_cargo_target_dir()?.into_string());
        }

        if build_skipped_when_env_is.0.is_empty() {
            build_skipped_when_env_is = vec![
                // shorter build for `cargo check`
                (crate::env_keys::RUST_PROFILE, "debug"),
                (crate::env_keys::BUILD_RS_ABI_STEP_HINT, "true"),
            ]
            .into();
        }

        let workdir = ManifestPath::get_manifest_workdir(build_opts.manifest_path.clone())?;

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
