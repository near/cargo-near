use cargo_metadata::DependencyKind;
use colored::Colorize;

use crate::types::{
    cargo::metadata::CrateMetadata, near::docker_build::metadata::ReproducibleBuild,
};
use std::{str::FromStr, time::Duration};

const MIN_CUTOFF: semver::Version = semver::Version::new(5, 2, 0);

// TODO #A: update to 5.12.0 after near-sdk-rs   pr release
const NEP330_1_3_0_CUTOFF: semver::Version = semver::Version::new(5, 11, 0);

pub fn suggest_near_sdk_checks(crate_metadata: &CrateMetadata) {
    match crate_metadata.find_direct_dependency("near_sdk") {
        Ok(packages) => {
            for package in packages {
                if package.0.version < MIN_CUTOFF {
                    println!(
                        "{}: {}",
                        "WARNING".truecolor(220, 77, 1),
                        "a `near-sdk` package version has been detected, which doesn't support reproducible builds at all!".red()
                    );
                    println!(
                        "{} < {}",
                        format!("{}", package.0.version).red(),
                        format!("{}", MIN_CUTOFF).cyan()
                    );
                    println!(
                        "{} {}",
                        "An upgrade recommended up to".red(),
                        format!("{}", MIN_CUTOFF).cyan()
                    );
                    std::thread::sleep(Duration::new(10, 0));
                    println!();
                } else if package.0.version < NEP330_1_3_0_CUTOFF {
                    println!(
                        "{}: {}",
                        "WARNING".red(),
                        "a `near-sdk` package version has been detected, which doesn't support latest reproducible builds NEP330 1.3.0 extension".yellow() // deep orange
                    );
                    println!(
                        "{} < {}",
                        format!("{}", package.0.version).yellow(),
                        format!("{}", NEP330_1_3_0_CUTOFF).cyan()
                    );
                    println!(
                        "{} {}",
                        "An upgrade recommended up to".yellow(),
                        format!("{}", NEP330_1_3_0_CUTOFF).cyan()
                    );
                    std::thread::sleep(Duration::new(2, 0));
                    println!();
                }
            }
        }
        Err(err) => {
            // we cannot return this error, as this warning is only a recommendation,
            // and isn't a showstopper
            println!("Encountered error when querying `near-sdk` dependency version");
            println!("{}", err);
        }
    }
}

pub const DOCKER_IMAGE_REGEX_PATTERN: &str = r#"^(?P<image>[^:@\s]+?)(?::(?P<tag>[^@\s]+?))?$"#;
pub const SOURCE_SCAN_TAG_PATTERN: &str = r#"^((?P<MAJOR>0|(?:[1-9]\d*))\.(?P<MINOR>0|(?:[1-9]\d*))\.(?P<PATCH>0|(?:[1-9]\d*)))-rust-((?P<MAJOR2>0|(?:[1-9]\d*))\.(?P<MINOR2>0|(?:[1-9]\d*))\.(?P<PATCH2>0|(?:[1-9]\d*)))$"#;

mod output_wasm_path {
    use colored::Colorize;
    use std::time::Duration;
    const CARGO_NEAR_BUILD_MIN: semver::Version = semver::Version::new(0, 5, 0);
    const CARGO_NEAR_MIN: semver::Version = semver::Version::new(0, 14, 0);

    pub fn versions_check(cargo_near: semver::Version, build_script: semver::Version) {
        match (
            cargo_near >= CARGO_NEAR_MIN,
            build_script >= CARGO_NEAR_BUILD_MIN,
        ) {
            (true, true) => {}
            (false, false) => {}
            (true, false) => {
                println!(
                        "{}: {}",
                        "WARNING".red(),
                        "incompatible versions of `cargo-near(docker image)` and `cargo-near-build(build-dependencies)` have been detected: addition of `output_wasm_path` field to BuildInfo".yellow() // deep orange
                    );
                println!("{}", "Reproducible build verification of product contracts, deployed from such factories, won't be successfull.".yellow());
                println!(
                    "cargo-near(docker image)            : {} >= {}",
                    format!("{}", cargo_near).yellow(),
                    format!("{}", CARGO_NEAR_MIN).cyan()
                );
                println!(
                    "cargo-near-build(build-dependencies): {} < {}",
                    format!("{}", build_script).yellow(),
                    format!("{}", CARGO_NEAR_BUILD_MIN).cyan()
                );
                println!(
                    "{} {}",
                    "An upgrade of `cargo-near-build(build-dependencies)` is recommended up to"
                        .yellow(),
                    format!("{}", CARGO_NEAR_BUILD_MIN).cyan()
                );
                println!();
                std::thread::sleep(Duration::new(2, 0));
            }
            (false, true) => {
                println!(
                        "{}: {}",
                        "WARNING".red(),
                        "incompatible versions of `cargo-near(docker image)` and `cargo-near-build(build-dependencies)` have been detected (addition of `output_wasm_path` field to BuildInfo)".yellow() // deep orange
                    );
                println!("{}", "Reproducible build verification of product contracts, deployed from such factories, won't be successfull.".yellow());
                println!(
                    "cargo-near(docker image)            : {} < {}",
                    format!("{}", cargo_near).yellow(),
                    format!("{}", CARGO_NEAR_MIN).cyan()
                );
                println!(
                    "cargo-near-build(build-dependencies): {} >= {}",
                    format!("{}", build_script).yellow(),
                    format!("{}", CARGO_NEAR_BUILD_MIN).cyan()
                );
                println!(
                    "{} {}",
                    "An upgrade of `cargo-near(docker image)` is recommended up to".yellow(),
                    format!("{}", CARGO_NEAR_MIN).cyan()
                );
                println!();
                std::thread::sleep(Duration::new(2, 0));
            }
        }
    }
}

pub fn suggest_cargo_near_build_checks(
    crate_metadata: &CrateMetadata,
    reproducible_build: &ReproducibleBuild,
) {
    let cargo_near_version = find_cargo_near_in_docker_img_tag(reproducible_build);
    let build_script_cargo_version = find_cargo_near_build_build_dep(crate_metadata);

    match (cargo_near_version, build_script_cargo_version) {
        (Some(cargo_near), Some(build_script)) => {
            output_wasm_path::versions_check(cargo_near, build_script);
        }
        _ => {}
    }
}

const PROD_IMAGE: &str = "sourcescan/cargo-near";
const DEV_IMAGE: &str = "dj8yfo/sourcescan";

fn find_cargo_near_in_docker_img_tag(
    reproducible_build: &ReproducibleBuild,
) -> Option<semver::Version> {
    let regex = regex::Regex::new(DOCKER_IMAGE_REGEX_PATTERN).expect("no error");
    let image = regex
        .captures(&reproducible_build.image)
        .and_then(|captures| captures.name("image"));

    let Some(image_match) = image else {
        return None;
    };
    if image_match.as_str() != PROD_IMAGE && image_match.as_str() != DEV_IMAGE {
        return None;
    }

    let tag = regex
        .captures(&reproducible_build.image)
        .and_then(|captures| captures.name("tag"));

    let Some(tag_match) = tag else { return None };

    let regex2 = regex::Regex::new(SOURCE_SCAN_TAG_PATTERN).expect("no error");
    let cargo_near_version = regex2
        .captures(tag_match.as_str())
        .and_then(|captures| captures.get(1));

    let Some(cargo_near_semver_match) = cargo_near_version else {
        return None;
    };
    semver::Version::from_str(cargo_near_semver_match.as_str()).ok()
}

fn find_cargo_near_build_build_dep(crate_metadata: &CrateMetadata) -> Option<semver::Version> {
    match crate_metadata.find_direct_dependency("cargo_near_build") {
        Ok(packages) => {
            let maybe_package = packages
                .into_iter()
                .find(|(_package, dep_kinds)| {
                    dep_kinds
                        .into_iter()
                        .find(|dep_kind_info| dep_kind_info.kind == DependencyKind::Build)
                        .is_some()
                })
                .map(|(package, _dep_kinds)| package);

            maybe_package.map(|pkg| pkg.version.clone())
        }
        Err(err) => {
            // we cannot return this error, as this warning is only a recommendation,
            // and isn't a showstopper
            println!("Encountered error when querying `cargo_near_build` dependency version");
            println!("{}", err);
            None
        }
    }
}
