use serde_json::json;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_docker_build() -> Result<(), Box<dyn std::error::Error>> {
    use cargo_near_integration_tests::setup_tracing;

    setup_tracing();

    let generated_manifest = {
        let generated_dir = run_cargo_near_new()?;
        generated_dir.join("Cargo.toml")
    };

    let opts = cargo_near_build::docker::DockerBuildOpts::builder()
        .manifest_path(generated_manifest.clone())
        .context(cargo_near_build::BuildContext::Build)
        .build();

    let artifact = cargo_near_build::docker::build(opts)?;

    let contract_wasm = std::fs::read(artifact.path)?;

    test_basics_on(&contract_wasm).await?;

    std::fs::remove_dir_all(
        generated_manifest
            .parent()
            .expect("expected to have parent"),
    )?;

    Ok(())
}

include! {"../../cargo-near/src/commands/new/test_basics_on.rs.in"}

#[test]
fn test_new_cmd_workspaces_toolchain_version() -> cargo_near::CliResult {
    let tests_manifest = {
        let cargo_near_integration_tests_dir: camino::Utf8PathBuf =
            env!("CARGO_MANIFEST_DIR").into();
        cargo_near_integration_tests_dir.join("Cargo.toml")
    };
    let generated_manifest = {
        let generated_dir = run_cargo_near_new()?;
        generated_dir.join("Cargo.toml")
    };

    let versions = [&tests_manifest, &generated_manifest]
        .iter()
        .map(|manifest| get_workspaces_rs_version(manifest))
        .collect::<Result<Vec<_>, color_eyre::Report>>()?;

    // This ensures sync of versions in source code of tests and `new` generated template
    assert_eq!(versions[0], versions[1]);

    // TODO: add sync of versions of rust-toolchain.toml-s

    std::fs::remove_dir_all(
        generated_manifest
            .parent()
            .expect("expected to have parent"),
    )?;
    Ok(())
}

fn get_workspaces_rs_version(manifest_path: &camino::Utf8PathBuf) -> color_eyre::Result<String> {
    use color_eyre::eyre::OptionExt;

    let toml_table_str = {
        let bytes = std::fs::read(manifest_path).map_err(|err| {
            color_eyre::eyre::eyre!("read file, {:?}, err {}", manifest_path, err)
        })?;
        core::str::from_utf8(&bytes)?.to_owned()
    };
    let toml_table = toml_table_str.parse::<toml::Table>()?;
    let entry = toml_table["dev-dependencies"]["near-workspaces"].clone();
    let result = match entry {
        toml::Value::String(version_string) => version_string,
        toml::Value::Table(workspaces_table) => workspaces_table["version"]
            .as_str()
            .ok_or_eyre(format!("expected version string {:#?}", workspaces_table))?
            .to_owned(),
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "unexpected variant of toml.dev-dependencies.near-workspaces"
            ))
        }
    };
    Ok(result)
}

fn run_cargo_near_new() -> color_eyre::Result<camino::Utf8PathBuf> {
    let out_path = {
        let tmp_dir = tempfile::Builder::new()
            .prefix("cargo_near_new_")
            .tempdir()?;
        tmp_dir.path().to_owned()
    };

    let scope = cargo_near::commands::new::InteractiveClapContextScopeForNew {
        project_dir: out_path.clone().into(),
    };
    let _result = cargo_near::commands::new::NewContext::from_previous_context(
        cargo_near::GlobalContext {
            config: Default::default(),
            offline: false,
            teach_me: false,
        },
        &scope,
    )?;
    Ok(out_path.try_into()?)
}
