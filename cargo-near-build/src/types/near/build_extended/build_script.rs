#[derive(Debug, Clone)]
pub struct Opts<'a> {
    /// environment variable name to export result `*.wasm` path to with [`cargo::rustc-env=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env)
    /// instruction
    pub result_env_key: Option<&'a str>,
    /// list of paths for [`cargo::rerun-if-changed=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
    /// instruction
    ///
    /// if relative, it's relative to path of crate, where build.rs is compiled
    pub rerun_if_changed_list: Vec<&'a str>,
    /// vector of key-value pairs of environment variable name and its value,
    /// when compilation should be skipped on a variable's value match;
    /// e.g.
    /// skipping emitting output `*.wasm` may be helpful when `PROFILE` is equal to `debug`
    /// for using  `rust-analyzer/flycheck`, `cargo check`, `bacon` and other dev-tools
    pub build_skipped_when_env_is: Vec<(&'a str, &'a str)>,
    /// path of stub file, where a placeholder empty `wasm` output is emitted to, when
    /// build is skipped due to match in [`Self::build_skipped_when_env_is`]
    ///
    /// if this path is relative, then the base is [`crate::extended::BuildOptsExtended::workdir`]
    pub stub_path: Option<&'a str>,
    /// substitution export of [`CARGO_TARGET_DIR`](https://doc.rust-lang.org/cargo/reference/environment-variables.html),
    /// which is required to avoid deadlock <https://github.com/rust-lang/cargo/issues/8938>;
    /// should best be a subfolder of [`CARGO_TARGET_DIR`](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
    /// of crate being built to work normally in docker builds
    ///
    /// if this path is relative, then the base is [`crate::extended::BuildOptsExtended::workdir`]
    pub distinct_target_dir: Option<&'a str>,
}
