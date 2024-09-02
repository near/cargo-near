use camino::{Utf8Path, Utf8PathBuf};
use eyre::WrapErr;

/// Copy a file to a destination.
///
/// Does nothing if the destination is the same as the source to avoid truncating the file.
pub fn copy(from: &Utf8Path, to: &Utf8Path) -> eyre::Result<Utf8PathBuf> {
    let out_path = to.join(from.file_name().unwrap());
    if from != out_path {
        std::fs::copy(from, &out_path)
            .wrap_err_with(|| format!("failed to copy `{}` to `{}`", from, out_path))?;
    }
    Ok(out_path)
}

/// Create the directory if it doesn't exist, and return the absolute path to it.
pub fn force_canonicalize_dir(dir: &Utf8Path) -> eyre::Result<Utf8PathBuf> {
    std::fs::create_dir_all(dir)
        .wrap_err_with(|| format!("failed to create directory `{}`", dir))?;
    // use canonicalize from `dunce` create instead of default one from std because it's compatible with Windows UNC paths
    // and don't break cargo compilation on Windows
    // https://github.com/rust-lang/rust/issues/42869
    Utf8PathBuf::from_path_buf(
        dunce::canonicalize(dir)
            .wrap_err_with(|| format!("failed to canonicalize path: {} ", dir))?,
    )
    .map_err(|err| eyre::eyre!("failed to convert path {}", err.to_string_lossy()))
}
