use std::env;
use std::ffi::OsString;
use std::fs;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;
use std::process;
use std::process::{Command, Stdio};

use anyhow::Context;

pub fn run(args: Vec<OsString>) -> ! {
    // https://github.com/rust-lang/rust-analyzer/blob/099f911b4ac0ebf5c375215b030ebf8800630bbb/crates/rust-analyzer/src/bin/main.rs#L30-L36
    let code = match invoke_rustc(args.into_iter()) {
        Ok(code) => code.unwrap_or(102),
        Err(err) => {
            eprintln!("{:?}", err);
            101
        }
    };
    process::exit(code)
}

pub(crate) fn invoke_rustc<I>(mut args: I) -> anyhow::Result<Option<i32>>
where
    I: Iterator<Item = OsString>,
{
    let rustc = args
        .next()
        .and_then(|c| c.into_string().ok())
        .unwrap_or_else(|| std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));

    let mut cmd = Command::new(rustc);

    cmd.args(args);

    if let (Ok(manifest_dir), Ok(abi_source_hash)) = (
        env::var("CARGO_MANIFEST_DIR"),
        env::var("CARGO_NEAR_ABI_SOURCE_HASH"),
    ) {
        let mut hash_state = std::collections::hash_map::DefaultHasher::new();
        manifest_dir.hash(&mut hash_state);
        if let Ok(source_hash) = u64::from_str_radix(&abi_source_hash, 16) {
            if source_hash == hash_state.finish() {
                cmd.args(["--cfg", "abi_embed"]);
            }
        }
    }
    if let (Ok("1"), Ok(abi_path), Ok(ack)) = (
        env::var("CARGO_PRIMARY_PACKAGE")
            .as_ref()
            .map(|s| s.as_str()),
        env::var("CARGO_NEAR_ABI_PATH"),
        env::var("CARGO_NEAR_ABI_ACK"),
    ) {
        let mut ack_path = PathBuf::from(abi_path);
        if ack_path.set_extension("ack") {
            fs::write(ack_path, ack)?;
        }

        log::info!("Invoking rustc: {:?}", cmd);
    }

    let mut child = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context(format!("Error executing `{:?}`", cmd))?;

    Ok(child.wait()?.code())
}
