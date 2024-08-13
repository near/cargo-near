// TODO: make module non-pub
pub mod target;

// TODO: make non-pub
pub const fn dylib_extension() -> &'static str {
    #[cfg(target_os = "linux")]
    return "so";

    #[cfg(target_os = "macos")]
    return "dylib";

    #[cfg(target_os = "windows")]
    return "dll";

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    compile_error!("Unsupported platform");
}
