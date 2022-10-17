use cargo_metadata::MetadataCommand;

fn main() -> anyhow::Result<()> {
    let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let meta = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .current_dir(&path)
        .exec()
        .unwrap();

    let root_node = meta
        .resolve
        .as_ref()
        .and_then(|dep_graph| {
            dep_graph
                .nodes
                .iter()
                .find(|node| node.id == meta.root_package().unwrap().id)
        })
        .ok_or_else(|| anyhow::anyhow!("unable to appropriately resolve the dependency graph"))?;

    let near_abi_dep = root_node
        .deps
        .iter()
        .find(|dep| dep.name == "near_abi")
        .and_then(|near_abi| meta.packages.iter().find(|pkg| pkg.id == near_abi.pkg))
        .ok_or_else(|| anyhow::anyhow!("`near-abi` dependency not found"))?;

    println!(
        "cargo:rustc-env=CARGO_NEAR_ABI_VERSION={}",
        &near_abi_dep.version
    );
    Ok(())
}
