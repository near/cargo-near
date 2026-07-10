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
    /// Emit a machine-readable JSON array of build jobs (suitable for a CI build matrix)
    ///
    /// One entry is emitted per (contract, variant) pair, so the array can be fed
    /// straight into a GitHub Actions `matrix` via `fromJSON(...)`.
    #[interactive_clap(long)]
    #[interactive_clap(verbatim_doc_comment)]
    pub json: bool,
}

impl From<CliListOpts> for ListOpts {
    fn from(value: CliListOpts) -> Self {
        Self {
            manifest_path: value.manifest_path,
            json: value.json,
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
            };
            super::run(opts)?;
            Ok(Self)
        }
    }
}

/// A single unit of work: build `package` (optionally with `variant`) and write it to `output`.
///
/// This is the JSON row shape a CI matrix consumes — each job builds exactly one wasm. It mirrors
/// [`cargo_near_build::list::BuildUnit`], with `manifest_path` stringified for the wire.
#[derive(Debug, serde::Serialize)]
pub struct BuildJob {
    /// Cargo package name of the contract crate, e.g. `defuse-poa-factory`.
    pub package: String,
    /// The named `reproducible_build` variant to build, or `null` for the default one.
    pub variant: Option<String>,
    /// Intended, collision-free output filename in the out-dir, e.g. `defuse.far.wasm`.
    pub output: String,
    /// Absolute path to the contract crate's `Cargo.toml`.
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

fn print_human_readable(contracts: &[cargo_near_build::list::WorkspaceContract]) {
    for contract in contracts {
        println!(
            "{} ({})",
            contract.name.bold(),
            contract.manifest_path.as_str().dimmed()
        );
        for variant in &contract.variants {
            let label = match variant {
                None => "<default>".to_string(),
                Some(name) => name.clone(),
            };
            println!(
                "  {} {}  →  {}",
                "•".cyan(),
                label,
                contract.output_filename(variant.as_deref())
            );
        }
    }
}

pub fn run(opts: ListOpts) -> color_eyre::eyre::Result<()> {
    let contracts =
        cargo_near_build::list::list_contracts(opts.manifest_path.as_ref().map(|p| p.as_path()))?;

    if opts.json {
        let jobs: Vec<BuildJob> = contracts
            .iter()
            .flat_map(|contract| contract.build_units())
            .map(BuildJob::from)
            .collect();
        println!("{}", serde_json::to_string_pretty(&jobs)?);
    } else if contracts.is_empty() {
        eprintln!(
            "{}",
            "No contracts with `[package.metadata.near.reproducible_build]` \
             were found in this workspace."
                .yellow()
        );
    } else {
        print_human_readable(&contracts);
    }

    Ok(())
}
