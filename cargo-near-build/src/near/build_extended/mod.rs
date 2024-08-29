mod build_script;
mod tmp_env;
use crate::types::near::build::output::CompilationArtifact;
use crate::types::near::build_extended::OptsExtended;
use rustc_version::Version;

pub fn run(args: OptsExtended) -> Result<CompilationArtifact, Box<dyn std::error::Error>> {
    let actual_version = rustc_version::version()?;
    let (artifact, skipped) = args.skip_or_compile(&actual_version)?;

    args.build_script_opts
        .post_build(skipped, &artifact, args.workdir, &actual_version)?;
    Ok(artifact)
}

impl<'a> OptsExtended<'a> {
    pub(crate) fn skip_or_compile(
        &self,
        version: &Version,
    ) -> Result<(CompilationArtifact, bool), Box<dyn std::error::Error>> {
        let _tmp_workdir = tmp_env::set_current_dir(self.workdir)?;
        let result = if self.build_script_opts.should_skip(version) {
            let artifact = self.build_script_opts.create_empty_stub()?;
            (artifact, true)
        } else {
            let artifact = self.compile_near_artifact()?;
            (artifact, false)
        };
        Ok(result)
    }

    /// `CARGO_TARGET_DIR` export is needed to avoid attempt to acquire same `target/<profile-path>/.cargo-lock`
    /// as the `cargo` process, which is running the build-script
    pub(crate) fn compile_near_artifact(
        &self,
    ) -> Result<CompilationArtifact, Box<dyn std::error::Error>> {
        let mut tmp_envs = vec![];
        for (env_key, value) in self.env.iter() {
            let tmp_env = tmp_env::set_var(env_key, value);
            tmp_envs.push(tmp_env);
        }

        let artifact = if let Some(distinct_target_dir) = self.build_script_opts.distinct_target_dir
        {
            let _tmp_cargo_target_env = tmp_env::set_var("CARGO_TARGET_DIR", distinct_target_dir);

            crate::build(self.build_opts.clone())?
        } else {
            crate::build(self.build_opts.clone())?
        };

        Ok(artifact)
    }
}
