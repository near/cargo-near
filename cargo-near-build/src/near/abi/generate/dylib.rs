use std::collections::HashSet;
use std::fs;

use camino::Utf8Path;

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
