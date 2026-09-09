use crate::types::cargo::metadata::CrateMetadata;
use crate::types::near::build::output::version_info::VersionInfo;

use super::{BuilderAbiVersions, Nep330BuildCommand, Nep330Link, Nep330Version};

/// NEP-330 build-time env vars describing the crate/build identity, independent of the emitted
/// wasm artifact. Shared verbatim between `cargo near build` and `cargo near check` so the two
/// commands can't silently type-check different configurations — add a new crate-level var here
/// once and both paths pick it up.
///
/// The artifact/output-tied vars (`NEP330_BUILD_INFO_OUTPUT_WASM_PATH` and the contract-path /
/// cargo-target-dir overrides) deliberately live on [`CommonVariables`](super::CommonVariables)
/// instead, since a `check` emits no artifact and exposes none of them.
pub struct Nep330CrateVars {
    pub nep330_version: Nep330Version,
    pub nep330_link: Nep330Link,
    pub nep330_build_cmd: Nep330BuildCommand,
    pub builder_abi_versions: BuilderAbiVersions,
}

impl Nep330CrateVars {
    /// `fallback_argv` supplies the `NEP330_BUILD_INFO_BUILD_COMMAND` argv in lib context (in cli
    /// context it's reconstructed from `std::env::args()` regardless): `build` passes its
    /// [`Opts::to_argv`](crate::BuildOpts), `check` passes its own `["cargo", "near", "check"]`.
    pub fn new(
        crate_metadata: &CrateMetadata,
        builder_version_info: &VersionInfo,
        fallback_argv: impl FnOnce() -> Vec<String>,
    ) -> eyre::Result<Self> {
        let nep330_version = Nep330Version::new(crate_metadata);
        let nep330_link = Nep330Link::new(crate_metadata);
        let nep330_build_cmd = Nep330BuildCommand::compute_with_fallback_argv(fallback_argv)?;
        let builder_abi_versions = builder_version_info.compute_env_variables()?;
        Ok(Self {
            nep330_version,
            nep330_link,
            nep330_build_cmd,
            builder_abi_versions,
        })
    }

    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        self.nep330_version.append_borrowed_to(env);
        self.nep330_link.append_borrowed_to(env);
        self.nep330_build_cmd.append_borrowed_to(env);
        self.builder_abi_versions.append_borrowed_to(env);
    }
}
