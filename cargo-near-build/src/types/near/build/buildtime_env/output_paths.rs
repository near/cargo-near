use crate::{env_keys, types::near::OutputPaths};

pub struct Nep330OutputWasmPath {
    pub value: String,
}

impl Nep330OutputWasmPath {
    pub fn new(output_paths: &OutputPaths) -> Self {
        Self {
            value: output_paths.wasm_file.to_string(),
        }
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push((env_keys::NEP330_OUTPUT_WASM_PATH, self.value.as_str()));
    }
}
