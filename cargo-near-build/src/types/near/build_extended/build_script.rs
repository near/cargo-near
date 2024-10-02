#[derive(Debug, Clone)]
pub struct Opts {
    /// environment variable name to export result `*.wasm` path to with [`cargo::rustc-env=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env)
    /// instruction
    pub result_env_key: Option<String>,
    /// list of paths for [`cargo::rerun-if-changed=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
    /// instruction
    ///
    /// if relative, it's relative to path of crate, where build.rs is compiled
    pub rerun_if_changed_list: Vec<String>,
    /// vector of key-value pairs of environment variable name and its value,
    /// when compilation should be skipped on a variable's value match;
    /// e.g.
    /// skipping emitting output `*.wasm` may be helpful when `PROFILE` is equal to `debug`
    /// for using  `rust-analyzer/flycheck`, `cargo check`, `bacon` and other dev-tools
    pub build_skipped_when_env_is: Vec<(String, String)>,
    /// path of stub file, where a placeholder empty `wasm` output is emitted to, when
    /// build is skipped due to match in [`Self::build_skipped_when_env_is`]
    ///
    /// if this path is relative, then the base is [`crate::extended::BuildOptsExtended::workdir`]
    pub stub_path: Option<String>,
}
