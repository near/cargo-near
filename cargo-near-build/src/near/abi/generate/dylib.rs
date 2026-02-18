use std::collections::HashSet;
use std::fs;

use camino::Utf8Path;
use eyre::WrapErr;

#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use crate::cargo_native::ArtifactType;
use crate::cargo_native::Dylib;
use crate::pretty_print;
use crate::types::near::build::output::CompilationArtifact;

pub fn extract_abi_entries(
    artifact: &CompilationArtifact<Dylib>,
) -> eyre::Result<Vec<near_abi::__private::ChunkedAbiEntry>> {
    let dylib_path: &Utf8Path = &artifact.path;
    let dylib_file_contents = fs::read(dylib_path)?;
    let object = symbolic_debuginfo::Object::parse(&dylib_file_contents)?;
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "A dylib was built at {:?} with format {} for architecture {}",
        &dylib_path,
        &object.file_format(),
        &object.arch()
    );
    let near_abi_symbols = object
        .symbols()
        .flat_map(|sym| sym.name)
        .filter(|sym_name| sym_name.starts_with("__near_abi_"))
        .collect::<HashSet<_>>();
    if near_abi_symbols.is_empty() {
        eyre::bail!("No NEAR ABI symbols found in the dylib");
    }
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Detected NEAR ABI symbols:\n{}",
        pretty_print::indent_payload(&format!("{:#?}", &near_abi_symbols))
    );

    // User-authored #[no_mangle] functions can reference NEAR host imports.
    // Those symbols are unresolved on host and would make dlopen fail before
    // we can call __near_abi_* export functions. Load a small shim library
    // with no-op definitions so ABI extraction can proceed.
    #[cfg(unix)]
    let _host_function_stubs = load_near_host_function_stubs()?;

    let mut entries = vec![];
    unsafe {
        let lib = libloading::Library::new(dylib_path.as_str())?;
        for symbol in near_abi_symbols {
            let entry: libloading::Symbol<extern "C" fn() -> (*const u8, usize)> =
                lib.get(symbol.as_bytes())?;
            let (ptr, len) = entry();
            let data = Vec::from_raw_parts(ptr as *mut _, len, len);
            match serde_json::from_slice(&data) {
                Ok(entry) => entries.push(entry),
                Err(err) => {
                    // unfortunately, we're unable to extract the raw error without Display-ing it first
                    let mut err_str = err.to_string();
                    if let Some((msg, rest)) = err_str.rsplit_once(" at line ") {
                        if let Some((line, col)) = rest.rsplit_once(" column ") {
                            if line.chars().all(|c| c.is_numeric())
                                && col.chars().all(|c| c.is_numeric())
                            {
                                err_str.truncate(msg.len());
                                err_str.shrink_to_fit();
                                eyre::bail!(err_str);
                            }
                        }
                    }
                    eyre::bail!(err);
                }
            };
        }
    }
    Ok(entries)
}

#[cfg(unix)]
struct LoadedHostFunctionStubs {
    _temp_dir: tempfile::TempDir,
    _library: libloading::os::unix::Library,
}

#[cfg(unix)]
fn load_near_host_function_stubs() -> eyre::Result<LoadedHostFunctionStubs> {
    use libloading::os::unix::{Library, RTLD_GLOBAL, RTLD_LAZY};

    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-near-abi-host-stubs")
        .tempdir()?;
    let source_path = temp_dir.path().join("near_host_stubs.rs");
    let library_path = temp_dir.path().join(format!(
        "libnear_host_stubs.{}",
        <Dylib as ArtifactType>::extension()
    ));

    fs::write(&source_path, near_host_stubs_source())?;

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(&rustc)
        .arg("--crate-name")
        .arg("near_host_stubs")
        .arg("--crate-type")
        .arg("cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&library_path)
        .output()
        .wrap_err_with(|| format!("failed to execute `{rustc}` while compiling ABI host stubs"))?;

    if !output.status.success() {
        eyre::bail!(
            "failed to compile ABI host stubs with `{}`:\n{}",
            rustc,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let library = unsafe { Library::open(Some(library_path.as_os_str()), RTLD_LAZY | RTLD_GLOBAL) }
        .wrap_err_with(|| {
            format!(
                "failed to load ABI host stubs from `{}`",
                library_path.display()
            )
        })?;

    Ok(LoadedHostFunctionStubs {
        _temp_dir: temp_dir,
        _library: library,
    })
}

#[cfg(unix)]
fn near_host_stubs_source() -> String {
    let mut source = String::from("#![allow(non_snake_case)]\n\n");
    for function in NEAR_HOST_FUNCTIONS {
        source.push_str("#[unsafe(no_mangle)]\n");
        source.push_str(&format!(
            "pub unsafe extern \"C\" fn {function}() -> u64 {{ 0 }}\n\n"
        ));
    }
    source
}

#[cfg(unix)]
const NEAR_HOST_FUNCTIONS: &[&str] = &[
    "read_register",
    "register_len",
    "write_register",
    "current_account_id",
    "current_contract_code",
    "refund_to_account_id",
    "signer_account_id",
    "signer_account_pk",
    "predecessor_account_id",
    "input",
    "block_index",
    "block_timestamp",
    "epoch_height",
    "storage_usage",
    "account_balance",
    "account_locked_balance",
    "attached_deposit",
    "prepaid_gas",
    "used_gas",
    "random_seed",
    "sha256",
    "keccak256",
    "keccak512",
    "ripemd160",
    "ecrecover",
    "ed25519_verify",
    "value_return",
    "panic",
    "panic_utf8",
    "log_utf8",
    "log_utf16",
    "abort",
    "promise_create",
    "promise_then",
    "promise_and",
    "promise_batch_create",
    "promise_batch_then",
    "promise_set_refund_to",
    "promise_batch_action_state_init",
    "promise_batch_action_state_init_by_account_id",
    "set_state_init_data_entry",
    "promise_batch_action_create_account",
    "promise_batch_action_deploy_contract",
    "promise_batch_action_function_call",
    "promise_batch_action_function_call_weight",
    "promise_batch_action_transfer",
    "promise_batch_action_stake",
    "promise_batch_action_add_key_with_full_access",
    "promise_batch_action_add_key_with_function_call",
    "promise_batch_action_delete_key",
    "promise_batch_action_delete_account",
    "promise_batch_action_deploy_global_contract",
    "promise_batch_action_deploy_global_contract_by_account_id",
    "promise_batch_action_use_global_contract",
    "promise_batch_action_use_global_contract_by_account_id",
    "promise_yield_create",
    "promise_yield_resume",
    "promise_results_count",
    "promise_result",
    "promise_return",
    "storage_write",
    "storage_read",
    "storage_remove",
    "storage_has_key",
    "storage_iter_prefix",
    "storage_iter_range",
    "storage_iter_next",
    "validator_stake",
    "validator_total_stake",
    "alt_bn128_g1_multiexp",
    "alt_bn128_g1_sum",
    "alt_bn128_pairing_check",
    "bls12381_p1_sum",
    "bls12381_p2_sum",
    "bls12381_g1_multiexp",
    "bls12381_g2_multiexp",
    "bls12381_map_fp_to_g1",
    "bls12381_map_fp2_to_g2",
    "bls12381_pairing_check",
    "bls12381_p1_decompress",
    "bls12381_p2_decompress",
];
