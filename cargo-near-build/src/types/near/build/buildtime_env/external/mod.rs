use crate::types::near::build::input;
pub(super) mod cargo_target_dir;
pub(super) mod nep330_path;

pub struct ExternalEnv {
    pub cargo_target_path: Option<cargo_target_dir::CargoTargetDir>,
    pub nep330_contract_path: nep330_path::Nep330ContractPath,
}

impl From<Option<input::implicit_env::Opts>> for ExternalEnv {
    fn from(value: Option<input::implicit_env::Opts>) -> Self {
        let cargo_target_path = cargo_target_dir::CargoTargetDir::maybe_new(
            value
                .as_ref()
                .and_then(|value| value.cargo_target_dir.clone()),
        );

        let nep330_contract_path = nep330_path::Nep330ContractPath::maybe_new(
            value
                .as_ref()
                .and_then(|value| value.nep330_contract_path.clone()),
        );

        Self {
            cargo_target_path,
            nep330_contract_path,
        }
    }
}
