use crate::env_keys;

pub struct CargoTargetDir {
    path: String,
}

impl CargoTargetDir {
    pub fn maybe_new(path: Option<String>) -> Option<Self> {
        path.map(|path| Self { path })
    }

    pub fn entry(&self) -> (&'static str, &str) {
        (env_keys::CARGO_TARGET_DIR, self.path.as_str())
    }

    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push(self.entry());
    }
}
