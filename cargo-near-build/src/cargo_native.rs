// TODO: make module non-pub
pub mod compile;
// TODO: make module non-pub
pub mod target;

pub trait ArtifactType {
    fn extension() -> &'static str;
}

pub struct WASM;

impl ArtifactType for WASM {
    fn extension() -> &'static str {
        "wasm"
    }
}

pub struct DYLIB;

impl ArtifactType for DYLIB {
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
