use camino::{Utf8Path, Utf8PathBuf};
use eyre::WrapErr;

/// Copy a file to a destination.
///
/// Does nothing if the destination is the same as the source to avoid truncating the file.
pub fn copy(from: &Utf8Path, out_dir: &Utf8Path) -> eyre::Result<Utf8PathBuf> {
    if !out_dir.is_dir() {
        return Err(eyre::eyre!("`{}` should be a directory", out_dir));
    }
    let out_path = out_dir.join(from.file_name().unwrap());
    tracing::debug!("Copying file `{}` -> `{}`", from, out_path,);
    if from != out_path {
        std::fs::copy(from, &out_path)
            .wrap_err_with(|| format!("failed to copy `{}` to `{}`", from, out_path))?;
    }
    Ok(out_path)
}

/// Copy a file to a file destination.
///
/// Does nothing if the destination is the same as the source to avoid truncating the file.
pub fn copy_to_file(from: &Utf8Path, to: &Utf8Path) -> eyre::Result<Utf8PathBuf> {
    tracing::debug!("Copying file `{}` -> `{}`", from, to,);

    if from != to && to.is_file() {
        let from_content = std::fs::read(from)?;
        let to_content = std::fs::read(to)?;

        if from_content == to_content {
            tracing::debug!(
                "skipped copying file `{}` -> `{}` on identical contents",
                from,
                to,
            );
            return Ok(to.to_path_buf());
        }
    }
    if from != to {
        std::fs::copy(from, to)
            .wrap_err_with(|| format!("failed to copy `{}` to `{}`", from, to))?;
    }
    Ok(to.to_path_buf())
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
