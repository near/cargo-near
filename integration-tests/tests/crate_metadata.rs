/// finds version of package in a crate's Cargo.lock
fn get_locked_package_version(
    manifest_path: &camino::Utf8PathBuf,
    package_name: &str,
) -> color_eyre::Result<Vec<semver::Version>> {
    let meta = cargo_near_build::CrateMetadata::collect(
        manifest_path.clone().try_into()?,
        false,
        None,
        false,
    )?;

    let packages = meta.find_direct_dependency(package_name)?;

    Ok(packages
        .into_iter()
        .map(|pkg| pkg.0.version.clone())
        .collect())
}

/// This asserts sync of versions of *package_name* in lock-file of
/// project_one vs project_two
pub fn assert_versions_equal(
    manifest_one: &camino::Utf8PathBuf,
    manifest_two: &camino::Utf8PathBuf,
    package_name: &str,
) -> cargo_near::CliResult {
    let versions = [manifest_one, manifest_two]
        .iter()
        .map(|manifest| get_locked_package_version(manifest, package_name))
        .collect::<Result<Vec<_>, color_eyre::Report>>()?;

    assert_eq!(
        versions[0].len(),
        1,
        "exactly a single dependency is expected to be found {:#?}",
        versions[0]
    );
    assert_eq!(
        versions[1].len(),
        1,
        "exactly a single dependency is expected to be found {:#?}",
        versions[1]
    );
    assert_eq!(
        versions[0],
        versions[1],
        "no sync of versions of `{}` in lock-file of `{}` and \
        `{}` projects",
        package_name,
        manifest_one.as_str(),
        manifest_two.as_str(),
    );
    Ok(())
}
