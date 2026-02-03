use cargo_near_build::docker::BuildContext;

use cargo_near_build::BuildArtifact;
use cargo_near_build::docker;

#[derive(Debug, Default, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = cargo_near_build::docker::BuildContext)]
#[interactive_clap(output_context = context::Context)]
pub struct BuildOpts {
    /// Disable implicit `--locked` flag for all `cargo` commands, enabled by default (NOT RECOMMENDED, DEMO MODE)
    ///
    /// Default behaviour without this flag: `--locked` flag is passed to `cargo metadata`
    /// downstream command being called.
    /// Enabling this flag will disable passing `--locked` downstream, and makes build NOT reproducible.  
    ///
    /// Running a `cargo` command with `--locked` will fail, if
    /// 1. the contract's crate doesn't have a Cargo.lock file,
    ///    which locks in place the versions of all of the contract's dependencies
    ///    (and, recursively, dependencies of dependencies ...), or
    /// 2. if it has Cargo.lock file, but it needs to be updated (happens if Cargo.toml manifest was updated)
    #[interactive_clap(long)]
    #[interactive_clap(verbatim_doc_comment)]
    pub no_locked: bool,
    /// Set build profile
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub profile: Option<String>,
    /// Copy final artifacts (`contract.wasm`) to this directory
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    pub out_dir: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Path to the `Cargo.toml` manifest of the contract crate to build in a docker container
    ///
    /// If this argument is not specified, by default the `Cargo.toml` in current directory is assumed
    /// as the manifest of target crate to build.
    ///
    /// 1. Contract is built inside of a docker container with rust toolchain installed therein.
    /// 2. Build command inside of the docker container and other options are configured via [config in manifest](template-project-manifest).
    /// 3. Possible flags of build command inside of the docker container can be looked up with `--help` on respetive `cargo-near` version,
    ///    and on the container itself, e.g.:
    ///     ```bash
    ///     docker run sourcescan/cargo-near:0.13.4-rust-1.84.0 cargo near build non-reproducible-wasm --help
    ///     docker run sourcescan/cargo-near:0.11.0-rust-1.82.0 cargo near build --help
    ///     ```
    /// 4. See also [verification-guide](sourcescan-verification-guide).
    ///
    /// [template-project-manifest]: https://github.com/near/cargo-near/blob/main/cargo-near/src/commands/new/new-project-template/Cargo.template.toml#L14-L29
    /// [sourcescan-verification-guide]: https://github.com/SourceScan/verification-guide
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    #[interactive_clap(verbatim_doc_comment)]
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Whether to color output to stdout and stderr by printing ANSI escape sequences: auto, always, never
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_interactive_input)]
    pub color: Option<crate::types::color_preference_cli::ColorPreferenceCli>,
    /// Specify which named variant of `reproducible-wasm` you want to use from `Cargo.toml`
    ///
    /// This flag will allow you to replace settings specified in default
    /// `[package.metadata.near.reproducible_build]`
    /// with ones defined in
    /// `[package.metadata.near.reproducible_build.variant.<VARIANT>]`.
    ///
    ///
    /// Note that all fields in `.variant.<VARIANT>` table are optional and
    /// will override default ones only if they are defined.
    ///
    /// Example:
    /// You have developed a feature in your contract that should work only for testnet.
    /// You could add this to your `Cargo.toml` (assuming you have already defined `testnet=[]` under `[features]`):
    /// ```toml
    /// [package.metadata.near.reproducible_build.variant.testnet]
    /// container_build_command = [
    ///     "cargo",
    ///     "near",
    ///     "build",
    ///     "non-reproducible-wasm",
    ///     "--locked",
    ///     "--features",
    ///     "testnet",
    /// ]
    /// ```
    /// This will only replace `container_build_command` of original `reproducible_build` with
    /// one from `.variant.testnet`.
    ///
    /// See also [variant flag explanation](variant-feature-explanation).
    ///
    /// [variant-feature-explanation]: https://github.com/near/cargo-near/tree/main?tab=readme-ov-file#custom-reproducible-wasm-build-using---variant-name-flag
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    #[interactive_clap(verbatim_doc_comment)]
    pub variant: Option<String>,
}

impl From<CliBuildOpts> for BuildOpts {
    fn from(value: CliBuildOpts) -> Self {
        Self {
            no_locked: value.no_locked,
            out_dir: value.out_dir,
            manifest_path: value.manifest_path,
            color: value.color,
            variant: value.variant,
            profile: value.profile,
        }
    }
}

mod context {
    #[derive(Debug)]
    pub struct Context;

    impl Context {
        pub fn from_previous_context(
            previous_context: cargo_near_build::docker::BuildContext,
            scope: &<super::BuildOpts as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
        ) -> color_eyre::eyre::Result<Self> {
            let opts = super::BuildOpts {
                no_locked: scope.no_locked,
                out_dir: scope.out_dir.clone(),
                manifest_path: scope.manifest_path.clone(),
                color: scope.color.clone(),
                variant: scope.variant.clone(),
                profile: scope.profile.clone(),
            };
            super::run(opts, previous_context)?;
            Ok(Self)
        }
    }
}

/// this is more or less equivalent to
/// impl From<(BuildCommand, BuildContext)> for docker::DockerBuildOpts
/// which is not possible due to BuildContext being a non-local type to current (cli) crate
fn docker_opts_from(value: (BuildOpts, BuildContext)) -> docker::DockerBuildOpts {
    docker::DockerBuildOpts {
        no_locked: value.0.no_locked,
        out_dir: value.0.out_dir.map(Into::into),
        manifest_path: value.0.manifest_path.map(Into::into),
        color: value.0.color.map(Into::into),
        variant: value.0.variant,
        context: value.1,
    }
}

pub fn run(opts: BuildOpts, context: BuildContext) -> color_eyre::eyre::Result<BuildArtifact> {
    let docker_opts = docker_opts_from((opts, context));
    cargo_near_build::docker::build(docker_opts, false)
}
