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
    /// Path to the crate's `Cargo.toml`, relative to the workspace root ([`Workspace::root`]),
    /// e.g. `contracts/defuse/Cargo.toml`. Relative so it is portable across machines and CI
    /// runners; join it onto [`Workspace::root`] for an absolute path. In the rare case a relative
    /// path can't be computed (a member on a different filesystem root), it is left absolute.
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
    /// The wasm filename `cargo near build` writes to its out-dir for this contract.
    ///
    /// This matches the artifact-naming rule the build pipeline already uses
    /// (`CrateMetadata::formatted_package_name`): the package name with `-` replaced by `_`, plus
    /// a `.wasm` extension. It does **not** depend on the variant, since `--variant` changes the
    /// build inputs but not the output path, so every variant of a package writes the same
    /// filename. In a CI matrix that's harmless (each job has its own out-dir); collecting several
    /// variants into one directory would require renaming per job.
    pub fn wasm_filename(&self) -> String {
        format!(
            "{}.{}",
            self.name.replace('-', "_"),
            crate::types::near::EXPECTED_WASM_EXTENSION
        )
    }

    /// One [`BuildUnit`] per variant, i.e. one per build that has to run.
    pub fn build_units(&self) -> impl Iterator<Item = BuildUnit> + '_ {
        let output = self.wasm_filename();
        self.variants.iter().map(move |variant| BuildUnit {
            package: self.name.clone(),
            variant: variant.clone(),
            output: output.clone(),
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
    /// The wasm filename `cargo near build` writes to the out-dir, e.g. `defuse_poa_token.wasm`.
    /// Variant-independent (see [`WorkspaceContract::wasm_filename`]).
    pub output: String,
    /// Path to the contract crate's `Cargo.toml`, relative to the workspace root
    /// (see [`WorkspaceContract::manifest_path`]).
    pub manifest_path: Utf8PathBuf,
}

/// A Cargo workspace and the contracts in it that opt into reproducible builds.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Absolute path to the workspace root (the directory containing the workspace `Cargo.toml`).
    /// Each contract's [`WorkspaceContract::manifest_path`] is relative to this.
    pub root: Utf8PathBuf,
    /// Discovered contracts, sorted by package name.
    pub contracts: Vec<WorkspaceContract>,
    /// Workspace members that do **not** opt into reproducible builds (no
    /// `[package.metadata.near.reproducible_build]` section), by package name, sorted. These are
    /// the members left out of [`contracts`](Self::contracts); surfaced so callers can report what
    /// was skipped rather than silently dropping it.
    pub skipped: Vec<String>,
}

/// Discover every workspace member that opts into reproducible builds, with its variants.
///
/// `manifest_path` may point at the workspace root or at any member's `Cargo.toml`; `None` uses
/// the `Cargo.toml` in the current directory to locate the enclosing workspace. Members without a
/// `[package.metadata.near.reproducible_build]` section are left out of
/// [`Workspace::contracts`] and recorded in [`Workspace::skipped`] instead. Results are sorted by
/// package name, and each contract's variants are sorted, so the output is deterministic.
///
/// Each [`WorkspaceContract::manifest_path`] is returned relative to [`Workspace::root`] (which is
/// itself absolute), so the paths are portable while staying resolvable via `root.join(...)`. If a
/// relative path can't be computed (a member on a different filesystem root), that member's
/// `manifest_path` is left absolute.
///
/// Runs `cargo metadata --no-deps`: the dependency graph is not resolved, which keeps this fast
/// and avoids requiring an up-to-date `Cargo.lock`. Presence of the `reproducible_build` section
/// is the only requirement; its contents (image, digest, ...) are not validated here, so a
/// half-filled section still shows up rather than being silently dropped.
pub fn list_contracts(manifest_path: Option<&Utf8Path>) -> eyre::Result<Workspace> {
    let mut command = cargo_metadata::MetadataCommand::new();
    command.no_deps();
    if let Some(manifest_path) = manifest_path {
        command.manifest_path(manifest_path.as_std_path().to_path_buf());
    }
    let metadata = command
        .exec()
        .wrap_err("Failed to run `cargo metadata` to enumerate workspace contracts")?;
    let root = metadata.workspace_root.clone();

    // One pass over the members: each is either a contract (has a `reproducible_build` section) or
    // is recorded as skipped, so nothing is silently dropped.
    let mut contracts: Vec<WorkspaceContract> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    for package in metadata.workspace_packages() {
        let Some(reproducible_build) = package
            .metadata
            .get("near")
            .and_then(|near| near.get("reproducible_build"))
        else {
            skipped.push(package.name.as_str().to_string());
            continue;
        };

        let mut variants = vec![None];
        if let Some(variant_table) = reproducible_build
            .get("variant")
            .and_then(|v| v.as_object())
        {
            let mut names: Vec<String> = variant_table.keys().cloned().collect();
            names.sort();
            variants.extend(names.into_iter().map(Some));
        }

        // Relative to the workspace root. `diff_utf8_paths` can express `../` for members above the
        // root, and only returns `None` when the two paths share no common base (e.g. different
        // filesystem roots / Windows drive prefixes); fall back to the absolute path there.
        let manifest_path = pathdiff::diff_utf8_paths(&package.manifest_path, &root)
            .unwrap_or_else(|| package.manifest_path.clone());

        contracts.push(WorkspaceContract {
            name: package.name.as_str().to_string(),
            manifest_path,
            variants,
        });
    }

    contracts.sort_by(|a, b| a.name.cmp(&b.name));
    skipped.sort();
    Ok(Workspace {
        root,
        contracts,
        skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract(name: &str, variants: Vec<Option<String>>) -> WorkspaceContract {
        WorkspaceContract {
            name: name.to_string(),
            manifest_path: Utf8PathBuf::from(format!("{name}/Cargo.toml")),
            variants,
        }
    }

    #[test]
    fn wasm_filename_normalizes_hyphens_to_underscores() {
        // Matches `CrateMetadata::formatted_package_name` / the actual cargo-near artifact name.
        assert_eq!(
            contract("defuse", vec![None]).wasm_filename(),
            "defuse.wasm"
        );
        assert_eq!(
            contract("defuse-poa-token", vec![None]).wasm_filename(),
            "defuse_poa_token.wasm"
        );
    }

    #[test]
    fn build_units_cover_every_variant_default_first_sharing_one_output() {
        let contract = contract(
            "defuse-wallet",
            vec![
                None,
                Some("no-auth".to_string()),
                Some("webauthn".to_string()),
            ],
        );
        let units: Vec<_> = contract.build_units().collect();
        assert_eq!(units.len(), 3);

        // Default first, then the variants in the order given.
        let variants: Vec<Option<&str>> = units.iter().map(|u| u.variant.as_deref()).collect();
        assert_eq!(variants, [None, Some("no-auth"), Some("webauthn")]);

        // `--variant` doesn't change the output path, so every unit writes the same file.
        for unit in &units {
            assert_eq!(unit.package, "defuse-wallet");
            assert_eq!(unit.output, "defuse_wallet.wasm");
            assert_eq!(unit.manifest_path, contract.manifest_path);
        }
    }
}
