use crate::env_keys;

pub struct Nep330BuildCommand {
    value: String,
}

impl Nep330BuildCommand {
    pub fn new(value: String) -> Self {
        tracing::info!("{}={}", env_keys::nep330::BUILD_COMMAND, value);
        Self { value }
    }
    pub fn append_borrowed_to<'a>(&'a self, env: &mut Vec<(&str, &'a str)>) {
        env.push((env_keys::nep330::BUILD_COMMAND, self.value.as_str()));
    }
}
