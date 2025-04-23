use bon::bon;
use camino::Utf8PathBuf;
use eyre::Context;

#[derive(Debug, Clone)]
pub struct BuildOptsExtended {
    pub build_opts: crate::BuildOpts,
}

#[bon]
impl BuildOptsExtended {
    #[builder]
    pub fn new(mut build_opts: crate::BuildOpts) -> eyre::Result<Self> {
        if let None = build_opts.override_cargo_target_dir {
            build_opts.override_cargo_target_dir = Some(override_cargo_target_dir()?.into_string());
        }

        Ok(Self { build_opts })
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
