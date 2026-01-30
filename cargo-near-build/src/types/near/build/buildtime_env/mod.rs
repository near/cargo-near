mod abi_path;

mod link;
mod version;

mod command;
mod output_paths;

mod abi_builder_version;

mod overrides;

pub use abi_path::AbiPath;

pub use link::Nep330Link;
pub use version::Nep330Version;

pub use command::Nep330BuildCommand;
pub use output_paths::Nep330OutputWasmPath;

pub use abi_builder_version::BuilderAbiVersions;

pub use overrides::nep330_path::Nep330ContractPath;

use crate::BuildOpts;
use crate::types::cargo::metadata::CrateMetadata;
use crate::types::near::OutputPaths;

use super::common_buildtime_env::overrides::cargo_target_dir::CargoTargetDir;
use super::output::version_info::VersionInfo;

/// variables, common for both steps of build, abi-gen and wasm build
pub struct CommonVariables {
    pub nep330_version: Nep330Version,
    pub nep330_link: Nep330Link,
    pub nep330_build_cmd: Nep330BuildCommand,
    pub nep330_output_wasm_path: Nep330OutputWasmPath,
    pub builder_abi_versions: BuilderAbiVersions,
    pub override_nep330_contract_path: Nep330ContractPath,
    pub override_cargo_target_path: CargoTargetDir,
}

impl CommonVariables {
    pub fn new(
        opts: &BuildOpts,
        builder_version_info: &VersionInfo,
        crate_metadata: &CrateMetadata,
        override_cargo_target_path: CargoTargetDir,
        output_paths: &OutputPaths,
    ) -> eyre::Result<Self> {
        let nep330_version = Nep330Version::new(crate_metadata);
        let nep330_link = Nep330Link::new(crate_metadata);

        let nep330_build_cmd = Nep330BuildCommand::compute(opts)?;
        let builder_abi_versions = builder_version_info.compute_env_variables()?;
        let override_nep330_contract_path =
            Nep330ContractPath::maybe_new(opts.override_nep330_contract_path.clone());

        let nep330_output_wasm_path =
            Nep330OutputWasmPath::new(opts.override_nep330_output_wasm_path.clone(), output_paths);
        let result = Self {
            nep330_version,
            nep330_link,
            nep330_build_cmd,
            nep330_output_wasm_path,
            builder_abi_versions,
            override_nep330_contract_path,
            override_cargo_target_path,
        };
        Ok(result)
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        self.nep330_version.append_borrowed_to(env);
        self.nep330_link.append_borrowed_to(env);
        self.nep330_build_cmd.append_borrowed_to(env);
        self.nep330_output_wasm_path.append_borrowed_to(env);
        self.builder_abi_versions.append_borrowed_to(env);
        self.override_nep330_contract_path.append_borrowed_to(env);
        self.override_cargo_target_path.append_borrowed_to(env);
    }
}
