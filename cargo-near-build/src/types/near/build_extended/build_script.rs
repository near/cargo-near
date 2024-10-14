#[derive(Debug, Clone, bon::Builder)]
pub struct Opts {
    /// environment variable name to export result `*.wasm` path to with [`cargo::rustc-env=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env)
    /// instruction
    #[builder(into)]
    pub result_env_key: Option<String>,
    /// list of paths for [`cargo::rerun-if-changed=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
    /// instruction
    ///
    /// if relative, it's relative to path of crate, where build.rs is compiled
    #[builder(default)]
    pub rerun_if_changed_list: Vec<String>,
    /// vector of key-value pairs of environment variable name and its value,
    /// when compilation should be skipped on a variable's value match;
    ///
    /// e.g.
    /// skipping emitting output `*.wasm` may be helpful when `PROFILE` is equal to `debug`
    /// for using  `rust-analyzer/flycheck`, `cargo check`, `bacon` and other dev-tools
    #[builder(default, into)]
    pub build_skipped_when_env_is: EnvPairs,
    /// path of stub file, where a placeholder empty `wasm` output is emitted to, when
    /// build is skipped due to match in [`Self::build_skipped_when_env_is`]
    #[builder(into)]
    pub stub_path: Option<String>,
}

/// utility type which can be initialized with vector of 2-element tuples of literal strings,
/// by using [core::convert::Into]
/// like so: `vec![("key1", "value1"), ("key2", "value2")].into()`
#[derive(Default, Debug, Clone)]
pub struct EnvPairs(pub Vec<(String, String)>);

impl From<Vec<(&str, &str)>> for EnvPairs {
    fn from(value: Vec<(&str, &str)>) -> Self {
        let vector = value
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        Self(vector)
    }
}
