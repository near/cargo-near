use colored::Colorize;

#[derive(Debug, Default, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = cargo_near_build::docker::BuildContext)]
#[interactive_clap(output_context = context::Context)]
pub struct ListOpts {
    /// Path to the `Cargo.toml` of the workspace (or of any member of it)
    ///
    /// If this argument is not specified, the `Cargo.toml` in the current directory is used
    /// to locate the enclosing workspace.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    #[interactive_clap(verbatim_doc_comment)]
    pub manifest_path: Option<crate::types::utf8_path_buf::Utf8PathBuf>,
    /// Emit a machine-readable JSON object of build jobs (suitable for a CI build matrix)
    ///
    /// The `jobs` array can be fed straight into a GitHub Actions `matrix` via
    /// `fromJSON(...)`.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    #[interactive_clap(verbatim_doc_comment)]
    pub json: bool,
    /// Include every reproducible-build variant, not just each contract's default one
    ///
    /// Without this flag, one job per contract is emitted (the default variant). With it,
    /// one job per (contract, variant) pair is emitted.
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    #[interactive_clap(verbatim_doc_comment)]
    pub variants: bool,
}

impl From<CliListOpts> for ListOpts {
    fn from(value: CliListOpts) -> Self {
        Self {
            manifest_path: value.manifest_path,
            json: value.json,
            variants: value.variants,
        }
    }
}

mod context {
    #[derive(Debug)]
    pub struct Context;

    impl Context {
        pub fn from_previous_context(
            _previous_context: cargo_near_build::docker::BuildContext,
            scope: &<super::ListOpts as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
        ) -> color_eyre::eyre::Result<Self> {
            let opts = super::ListOpts {
                manifest_path: scope.manifest_path.clone(),
                json: scope.json,
                variants: scope.variants,
            };
            super::run(opts)?;
            Ok(Self)
        }
    }
}

/// A single unit of work: build `package` (optionally with `variant`) and write it to `output`.
///
/// This is the JSON row shape a CI matrix consumes; each job builds exactly one wasm. It mirrors
/// [`cargo_near_build::list::BuildUnit`], with `manifest_path` stringified for the wire.
#[derive(Debug, serde::Serialize)]
pub struct BuildJob {
    /// Cargo package name of the contract crate, e.g. `defuse-poa-factory`.
    pub package: String,
    /// The named `reproducible_build` variant to build, or `null` for the default one.
    pub variant: Option<String>,
    /// Wasm filename `cargo near build` writes to the out-dir, e.g. `defuse_poa_token.wasm`.
    /// Variant-independent, so a package's variants share this name.
    pub output: String,
    /// Path to the contract crate's `Cargo.toml`, relative to `workspace_root` (absolute only in
    /// the rare case a relative path can't be computed, e.g. a different filesystem root).
    pub manifest_path: String,
}

impl From<cargo_near_build::list::BuildUnit> for BuildJob {
    fn from(unit: cargo_near_build::list::BuildUnit) -> Self {
        Self {
            package: unit.package,
            variant: unit.variant,
            output: unit.output,
            manifest_path: unit.manifest_path.to_string(),
        }
    }
}

/// The `--json` output: a versioned envelope around the [`BuildJob`] rows.
///
/// Wrapping the jobs (rather than emitting a bare array) leaves room to add fields without a
/// breaking change; consumers can gate on `version`.
#[derive(Debug, serde::Serialize)]
struct ListOutput {
    /// Envelope schema version. Bumped only on a breaking change to this shape.
    version: u32,
    /// Absolute path to the workspace root; each job's `manifest_path` is relative to it (absolute
    /// only in the rare case a relative path can't be computed).
    workspace_root: String,
    /// The build jobs, one per emitted (contract, variant) pair.
    jobs: Vec<BuildJob>,
}

fn print_human_readable(workspace: &cargo_near_build::list::Workspace, variants: bool) {
    println!(
        "{} {}",
        "workspace:".dimmed(),
        workspace.root.as_str().dimmed()
    );
    for contract in &workspace.contracts {
        println!(
            "{} ({})  →  {}",
            contract.name.bold(),
            contract.manifest_path.as_str().dimmed(),
            contract.wasm_filename()
        );
        for variant in &contract.variants {
            // Without `--variants`, show only each contract's default variant.
            if !variants && variant.is_some() {
                continue;
            }
            let label = match variant {
                None => "<default>",
                Some(name) => name,
            };
            println!("  {} {}", "•".cyan(), label);
        }
    }
}

pub fn run(opts: ListOpts) -> color_eyre::eyre::Result<()> {
    let workspace =
        cargo_near_build::list::list_contracts(opts.manifest_path.as_ref().map(|p| p.as_path()))?;

    if opts.json {
        // Without `--variants`, emit one job per contract (the default variant only).
        let jobs: Vec<BuildJob> = workspace
            .contracts
            .iter()
            .flat_map(|contract| contract.build_units())
            .filter(|unit| opts.variants || unit.variant.is_none())
            .map(BuildJob::from)
            .collect();
        let output = ListOutput {
            version: 1,
            workspace_root: workspace.root.to_string(),
            jobs,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if workspace.contracts.is_empty() {
        eprintln!(
            "{}",
            "No contracts with `[package.metadata.near.reproducible_build]` \
             were found in this workspace."
                .yellow()
        );
    } else {
        print_human_readable(&workspace, opts.variants);
    }

    // Report skipped members on stderr (both modes) so stdout/jq stays clean.
    super::warn_skipped_members(&workspace);

    Ok(())
}
