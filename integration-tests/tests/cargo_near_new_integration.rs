#[cfg(target_os = "linux")]
#[test]
fn test_docker_build() -> cargo_near::CliResult {
    use cargo_near_integration_tests::setup_tracing;

    setup_tracing();
    let manifest_dir: camino::Utf8PathBuf = env!("CARGO_MANIFEST_DIR").into();

    let cargo_path_parent = manifest_dir.join("docker-build-template");
    let cargo_path = cargo_path_parent.join("Cargo.toml");

    let opts = cargo_near_build::docker::DockerBuildOpts::builder()
        .manifest_path(cargo_path.into())
        .build();

    cargo_near_build::docker::build(opts)?;

    Ok(())
}
