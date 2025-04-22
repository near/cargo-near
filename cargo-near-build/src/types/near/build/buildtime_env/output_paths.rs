use crate::{env_keys::nep330::OUTPUT_WASM_PATH, types::near::OutputPaths};

pub struct Nep330OutputWasmPath {
    pub value: String,
}

impl Nep330OutputWasmPath {
    /// priority of setting the value:
    /// 1. most priority in `api_override_value` arg if it's set to [Option::Some]
    ///   - currently this [Option::Some] branch isn't expected to be used anywhere, but it
    ///     was added (to public api) for symmetry with [crate::build_with_cli] api, where it's expected to be used
    /// 2. if external [crate::env_keys::nep330::OUTPUT_WASM_PATH] is set, it's used unchanged
    /// 3. and then, if none of the above happened, then value from `output_paths` arg is used
    pub fn new(api_override_value: Option<String>, output_paths: &OutputPaths) -> Self {
        match api_override_value {
            Some(value) => Self { value },
            None => {
                let value = if let Ok(env_override_value) = std::env::var(OUTPUT_WASM_PATH) {
                    env_override_value
                } else {
                    output_paths.wasm_file.to_string()
                };
                Self { value }
            }
        }
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push((OUTPUT_WASM_PATH, self.value.as_str()));
    }
}
