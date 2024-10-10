use std::fmt::Debug;
use std::path::Path;

/// Memorize the current path and switch to the given path. Once the datastructure is
/// dropped, switch back to the original path automatically.
pub fn set_current_dir<P: AsRef<Path>>(path: P) -> Result<CurrentDir, std::io::Error> {
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(&path)?;
    Ok(CurrentDir(current_dir))
}

/// A helper datastructure for ensuring that we switch back to the current folder before the
/// end of the current scope.
pub struct CurrentDir(std::path::PathBuf);

impl Debug for CurrentDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Drop for CurrentDir {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.0).expect("cannot go back to the previous directory");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_dir() {
        println!(
            "current dir: {:?}",
            std::env::current_dir().expect("patronus")
        );
        {
            let _tmp_current_dir = set_current_dir("src").expect("should set the new current_dir");
            let current_dir = std::env::current_dir().expect("cannot get current dir from std env");
            assert!(current_dir.ends_with("src"));
        }
        let current_dir = std::env::current_dir().expect("cannot get current dir from std env");
        assert!(!current_dir.ends_with("src"));
        // Because guard is dropped
        set_current_dir("src/near").expect("should set the new current_dir");
        assert!(!current_dir.ends_with("src/near"));
    }
}
