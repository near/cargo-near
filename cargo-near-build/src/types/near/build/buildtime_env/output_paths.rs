use crate::{env_keys::nep330::OUTPUT_WASM_PATH, types::near::OutputPaths};

pub struct Nep330OutputWasmPath {
    pub value: String,
}

impl Nep330OutputWasmPath {
    pub fn new(output_paths: &OutputPaths) -> Self {
        let value = if let Ok(external_value) = std::env::var(OUTPUT_WASM_PATH) {
            external_value
        } else {
            output_paths.wasm_file.to_string()
        };
        Self { value }
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push((OUTPUT_WASM_PATH, self.value.as_str()));
    }
}
