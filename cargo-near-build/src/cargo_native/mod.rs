#[cfg(feature = "build_internal")]
pub mod compile;
#[cfg(feature = "build_internal")]
pub mod target;

pub trait ArtifactType {
    fn extension() -> &'static str;
}

pub struct Wasm;

impl ArtifactType for Wasm {
    fn extension() -> &'static str {
        "wasm"
    }
}

pub struct Dylib;

impl ArtifactType for Dylib {
    fn extension() -> &'static str {
        #[cfg(target_os = "linux")]
        return "so";

        #[cfg(target_os = "macos")]
        return "dylib";

        #[cfg(target_os = "windows")]
        return "dll";

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        compile_error!("Unsupported platform");
    }
}
