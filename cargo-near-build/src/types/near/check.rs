/// Which cargo subcommand the `check` path drives.
///
/// Both run under the exact same environment `cargo near build` uses (`--cfg near`,
/// `wasm32-unknown-unknown` target, same feature/profile/locked resolution and toolchain),
/// but neither produces a wasm artifact.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CheckKind {
    /// `cargo check` — fast type-check, the default.
    #[default]
    Check,
    /// `cargo clippy` — type-check plus clippy lints.
    Clippy,
}

impl CheckKind {
    /// The cargo subcommand string driven for this kind.
    pub(crate) fn cargo_subcommand(self) -> &'static str {
        match self {
            CheckKind::Check => "check",
            CheckKind::Clippy => "clippy",
        }
    }
}

/// Argument of [`check`](crate::check).
///
/// Mirrors the subset of [`BuildOpts`](crate::BuildOpts) that affects which configuration is
/// type-checked. Build-only fields (`no_abi`/`no_embed_abi`/`no_doc`/`no_wasmopt`/`out_dir`/
/// the NEP330 `override_*` outputs/`skip_rust_version_check`) are intentionally absent — a
/// type-check emits no wasm, so ABI generation, `wasm-opt`, output copying and the
/// rustc/protocol-version ceiling check don't apply.
///
/// [`std::default::Default`] yields a `cargo check` (not clippy) of the current directory's
/// contract, `--release`, `--locked`.
#[derive(Debug, Default, Clone, bon::Builder)]
pub struct Opts {
    /// Run `cargo clippy` instead of `cargo check`.
    #[builder(default)]
    pub clippy: bool,
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    #[builder(default)]
    pub no_locked: bool,
    /// Type-check in debug mode instead of `--release`
    #[builder(default)]
    pub no_release: bool,
    /// Set build profile
    pub profile: Option<String>,
    /// Path to the `Cargo.toml` of the contract to check
    #[builder(into)]
    pub manifest_path: Option<camino::Utf8PathBuf>,
    /// Set compile-time feature flags.
    #[builder(into)]
    pub features: Option<String>,
    /// Disables default feature flags.
    #[builder(default)]
    pub no_default_features: bool,
    /// Coloring: auto, always, never;
    /// assumed to be auto when `None`
    pub color: Option<crate::types::near::build::input::ColorPreference>,
    /// additional environment key-value pairs, that should be passed to the underlying
    /// `cargo check`/`cargo clippy` command
    #[builder(default)]
    pub env: Vec<(String, String)>,
    /// override value of [`crate::env_keys::RUSTUP_TOOLCHAIN`] environment variable, used for
    /// all invoked `rustc`, `cargo` and `rustup` commands
    #[builder(into)]
    pub override_toolchain: Option<String>,
}
