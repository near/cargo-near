//! Enumerate the NEAR contracts in a Cargo workspace and their reproducible-build variants.
//!
//! This is the discovery primitive behind `cargo near build list`. It is deliberately
//! dependency-light (just `cargo metadata --no-deps`), so build scripts and test harnesses
//! such as `near-workspaces` can reuse it to drive multi-contract builds, e.g. one CI matrix
//! job per (contract, variant) pair.

use camino::{Utf8Path, Utf8PathBuf};
use eyre::WrapErr;

/// A workspace member that opts into reproducible builds via
/// `[package.metadata.near.reproducible_build]`, together with its build variants.
#[derive(Debug, Clone)]
pub struct WorkspaceContract {
    /// Cargo package name, e.g. `defuse-poa-factory`.
    pub name: String,
    /// Absolute path to the crate's `Cargo.toml`.
    pub manifest_path: Utf8PathBuf,
    /// Build variants, default first.
    ///
    /// `None` is the default (unnamed) variant, i.e. the top-level
    /// `[package.metadata.near.reproducible_build]` table. Each `Some(name)` is a
    /// `[package.metadata.near.reproducible_build.variant.<name>]` sub-table. Named variants
    /// are sorted for deterministic output.
    pub variants: Vec<Option<String>>,
}

impl WorkspaceContract {
    /// Collision-free wasm filename for one of this contract's variants.
    ///
    /// The default variant is `{name}.wasm`; a named variant is `{name}.{variant}.wasm`, so two
    /// variants of the same crate don't clobber each other when collected into a single out-dir.
    pub fn output_filename(&self, variant: Option<&str>) -> String {
        match variant {
            None => format!("{}.wasm", self.name),
            Some(variant) => format!("{}.{variant}.wasm", self.name),
        }
    }

    /// One [`BuildUnit`] per variant, i.e. one per build that has to run.
    pub fn build_units(&self) -> impl Iterator<Item = BuildUnit> + '_ {
        self.variants.iter().map(move |variant| BuildUnit {
            package: self.name.clone(),
            variant: variant.clone(),
            output: self.output_filename(variant.as_deref()),
            manifest_path: self.manifest_path.clone(),
        })
    }
}

/// A single build to run: one (contract, variant) pair. This is the row a CI matrix consumes,
/// each job building exactly one wasm.
#[derive(Debug, Clone)]
pub struct BuildUnit {
    /// Cargo package name of the contract crate.
    pub package: String,
    /// The named `reproducible_build` variant to build, or `None` for the default one.
    pub variant: Option<String>,
    /// Collision-free output filename in the out-dir, e.g. `defuse.far.wasm`.
    pub output: String,
    /// Absolute path to the contract crate's `Cargo.toml`.
    pub manifest_path: Utf8PathBuf,
}

/// Discover every workspace member that opts into reproducible builds, with its variants.
///
/// `manifest_path` may point at the workspace root or at any member's `Cargo.toml`; `None` uses
/// the `Cargo.toml` in the current directory to locate the enclosing workspace. Members without a
/// `[package.metadata.near.reproducible_build]` section are skipped. Results are sorted by package
/// name, and each contract's variants are sorted, so the output is deterministic.
///
/// Runs `cargo metadata --no-deps`: the dependency graph is not resolved, which keeps this fast
/// and avoids requiring an up-to-date `Cargo.lock`. Presence of the `reproducible_build` section
/// is the only requirement; its contents (image, digest, ...) are not validated here, so a
/// half-filled section still shows up rather than being silently dropped.
pub fn list_contracts(manifest_path: Option<&Utf8Path>) -> eyre::Result<Vec<WorkspaceContract>> {
    let mut command = cargo_metadata::MetadataCommand::new();
    command.no_deps();
    if let Some(manifest_path) = manifest_path {
        command.manifest_path(manifest_path.as_std_path().to_path_buf());
    }
    let metadata = command
        .exec()
        .wrap_err("Failed to run `cargo metadata` to enumerate workspace contracts")?;

    let mut contracts: Vec<WorkspaceContract> = metadata
        .workspace_packages()
        .into_iter()
        .filter_map(|package| {
            let reproducible_build = package
                .metadata
                .get("near")
                .and_then(|near| near.get("reproducible_build"))?;

            let mut variants = vec![None];
            if let Some(variant_table) = reproducible_build
                .get("variant")
                .and_then(|v| v.as_object())
            {
                let mut names: Vec<String> = variant_table.keys().cloned().collect();
                names.sort();
                variants.extend(names.into_iter().map(Some));
            }

            Some(WorkspaceContract {
                name: package.name.as_str().to_string(),
                manifest_path: package.manifest_path.clone(),
                variants,
            })
        })
        .collect();

    contracts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(contracts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract(name: &str, variants: Vec<Option<String>>) -> WorkspaceContract {
        WorkspaceContract {
            name: name.to_string(),
            manifest_path: Utf8PathBuf::from(format!("/ws/{name}/Cargo.toml")),
            variants,
        }
    }

    #[test]
    fn output_filename_default_and_named() {
        let contract = contract("defuse", vec![None, Some("far".to_string())]);
        assert_eq!(contract.output_filename(None), "defuse.wasm");
        assert_eq!(contract.output_filename(Some("far")), "defuse.far.wasm");
    }

    #[test]
    fn build_units_cover_every_variant_default_first() {
        let contract = contract(
            "wallet",
            vec![
                None,
                Some("no-auth".to_string()),
                Some("webauthn".to_string()),
            ],
        );
        let units: Vec<_> = contract.build_units().collect();
        assert_eq!(units.len(), 3);

        assert_eq!(units[0].variant, None);
        assert_eq!(units[0].output, "wallet.wasm");
        assert_eq!(units[0].package, "wallet");
        assert_eq!(units[0].manifest_path, contract.manifest_path);

        assert_eq!(units[1].variant.as_deref(), Some("no-auth"));
        assert_eq!(units[1].output, "wallet.no-auth.wasm");

        assert_eq!(units[2].variant.as_deref(), Some("webauthn"));
        assert_eq!(units[2].output, "wallet.webauthn.wasm");
    }
}
