use crate::{env_keys::nep330::OUTPUT_WASM_PATH, types::near::OutputPaths, BuildOpts};

pub struct Nep330OutputWasmPath {
    pub value: String,
}

impl Nep330OutputWasmPath {
    pub fn new(output_paths: &OutputPaths, opts: &BuildOpts) -> Self {
        let value = if let Some(ref override_value) = opts.override_output_wasm_path {
            override_value.clone()
        } else {
            output_paths.wasm_file.to_string()
        };
        Self { value }
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push((OUTPUT_WASM_PATH, self.value.as_str()));
    }
}
