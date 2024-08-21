use std::ffi::{OsStr, OsString};
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

/// Sets the environment variable k to the value v for the currently running process.
/// It returns a datastructure to keep the environment variable set. When dropped the environment variable is restored
pub fn set_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) -> CurrentEnv {
    let key = key.as_ref();
    let previous_val = std::env::var(key).ok();
    std::env::set_var(key, value);
    CurrentEnv(key.to_owned(), previous_val)
}

/// A helper datastructure for ensuring that we restore the current environment variable before the
/// end of the current scope.
pub struct CurrentEnv(OsString, Option<String>);

impl Debug for CurrentEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Drop for CurrentEnv {
    fn drop(&mut self) {
        match self.1.take() {
            Some(previous_val) => std::env::set_var(&self.0, previous_val),
            None => std::env::remove_var(&self.0),
        }
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

    #[test]
    fn test_env() {
        {
            let _tmp_env = set_var("TEST_TMP_ENV", "myvalue");
            assert_eq!(std::env::var("TEST_TMP_ENV"), Ok(String::from("myvalue")));
        }
        assert!(std::env::var("TEST_TMP_ENV").is_err());
        // Because guard is dropped
        set_var("TEST_TMP_ENV_DROPPED", "myvaluedropped");
        assert!(std::env::var("TEST_TMP_ENV_DROPPED").is_err());
    }

    #[test]
    fn test_env_with_previous_value() {
        std::env::set_var("TEST_TMP_ENV_PREVIOUS", "previous_value");
        {
            let _tmp_env = set_var("TEST_TMP_ENV_PREVIOUS", "myvalue");
            assert_eq!(
                std::env::var("TEST_TMP_ENV_PREVIOUS"),
                Ok(String::from("myvalue"))
            );
        }
        assert_eq!(
            std::env::var("TEST_TMP_ENV_PREVIOUS"),
            Ok(String::from("previous_value"))
        );
    }
}
