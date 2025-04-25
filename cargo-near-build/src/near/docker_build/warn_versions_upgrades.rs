use cargo_metadata::DependencyKind;
use colored::Colorize;

use crate::types::{
    cargo::metadata::CrateMetadata, near::docker_build::metadata::ReproducibleBuild,
};
use std::{str::FromStr, time::Duration};

const MIN_SDK_REPRODUCIBLE: cargo_metadata::semver::Version =
    cargo_metadata::semver::Version::new(5, 2, 0);

pub struct NearSdkFeatureSupport {
    pub output_wasm_path: bool,
}

pub fn suggest_near_sdk_checks(crate_metadata: &CrateMetadata) -> NearSdkFeatureSupport {
    let mut result = NearSdkFeatureSupport {
        output_wasm_path: false,
    };
    match crate_metadata.find_direct_dependency("near_sdk") {
        Ok(packages) => {
            for package in packages {
                let near_sdk_version = package.0.version.clone();
                if near_sdk_version < MIN_SDK_REPRODUCIBLE {
                    println!(
                        "{}: {}",
                        "WARNING".truecolor(220, 77, 1),
                        "a `near-sdk` package version has been detected, which doesn't support reproducible builds at all!".red()
                    );
                    println!(
                        "{} < {}",
                        format!("{}", near_sdk_version).red(),
                        format!("{}", MIN_SDK_REPRODUCIBLE).cyan()
                    );
                    println!(
                        "{} {}",
                        "An upgrade recommended up to".red(),
                        format!("{}", MIN_SDK_REPRODUCIBLE).cyan()
                    );
                    std::thread::sleep(Duration::new(10, 0));
                    println!();
                }
                let support = output_wasm_path::near_sdk_version_check(near_sdk_version);

                if support.0 {
                    result.output_wasm_path = true;
                }
            }
        }
        Err(err) => {
            // we cannot return this error, as this warning is only a recommendation,
            // and isn't a showstopper
            println!(
                "{}",
                "Encountered error when querying `near-sdk` dependency version".red()
            );
            println!("{}", err);
        }
    }
    result
}

pub const DOCKER_IMAGE_REGEX_PATTERN: &str = r#"^(?P<image>[^:@\s]+?)(?::(?P<tag>[^@\s]+?))?$"#;
pub const SOURCE_SCAN_TAG_PATTERN: &str = r#"^((?P<MAJOR>0|(?:[1-9]\d*))\.(?P<MINOR>0|(?:[1-9]\d*))\.(?P<PATCH>0|(?:[1-9]\d*)))-rust-((?P<MAJOR2>0|(?:[1-9]\d*))\.(?P<MINOR2>0|(?:[1-9]\d*))\.(?P<PATCH2>0|(?:[1-9]\d*)))$"#;

mod output_wasm_path {
    use colored::Colorize;
    use std::time::Duration;

    const CARGO_NEAR_BUILD_MIN: cargo_metadata::semver::Version =
        cargo_metadata::semver::Version::new(0, 6, 0);
    const CARGO_NEAR_MIN: cargo_metadata::semver::Version =
        cargo_metadata::semver::Version::new(0, 14, 0);
    const NEAR_SDK_MIN: cargo_metadata::semver::Version =
        cargo_metadata::semver::Version::new(5, 12, 0);
    pub struct NearSDKSupports(pub bool);

    pub fn near_sdk_version_check(near_sdk: cargo_metadata::semver::Version) -> NearSDKSupports {
        if near_sdk >= NEAR_SDK_MIN {
            return NearSDKSupports(true);
        }
        if near_sdk < NEAR_SDK_MIN && near_sdk >= super::MIN_SDK_REPRODUCIBLE {
            println!(
                        "{}: {} {} {}",
                        "INFO".truecolor(220, 77, 1),
                        "a".yellow(),
                        "near-sdk".cyan(),
                        "package version has been detected, which doesn't support latest reproducible builds NEP330 1.3.0 extension".yellow()
                    );

            println!(
                "{} < {}",
                format!("{}", near_sdk).yellow(),
                format!("{}", NEAR_SDK_MIN).cyan()
            );
            println!(
                "{} {}",
                "An upgrade recommended up to".yellow(),
                format!("{}", NEAR_SDK_MIN).cyan()
            );
            println!(
                        "{}",
                        "`near-sdk` upgrade is optional. Build is verifiable for WASM reproducibility without it.".cyan(),
                    );
            std::thread::sleep(Duration::new(2, 0));
            println!();
        }
        NearSDKSupports(false)
    }

    pub fn cargo_near_version_check(cargo_near: cargo_metadata::semver::Version) {
        if cargo_near < CARGO_NEAR_MIN {
            println!(
                        "{}: {} {} {}",
                        "INFO".truecolor(220, 77, 1),
                        "a".yellow(),
                        "[package.metadata.near.reproducible_build.image]".cyan(),
                        "docker image has been detected, which doesn't support latest reproducible builds NEP330 1.3.0 extension".yellow()
                    );
            println!(
                "{} < {}",
                format!("{}", cargo_near).yellow(),
                format!("{}", CARGO_NEAR_MIN).cyan()
            );
            println!(
                "{} {}",
                "An upgrade of docker image is recommended up to".yellow(),
                format!("{}", CARGO_NEAR_MIN).cyan()
            );
            println!(
                    "{}",
                    "docker image upgrade is optional. Build is verifiable for WASM reproducibility without it.".cyan(),
                );
            println!();
            std::thread::sleep(Duration::new(2, 0));
        }
    }
    pub fn with_buildscript_versions_check(
        cargo_near: cargo_metadata::semver::Version,
        build_script: cargo_metadata::semver::Version,
    ) {
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
                    "incompatible versions of `cargo-near(docker image)` and `cargo-near-build(build-dependencies)` have been detected: ".yellow()
                );
                println!(
                    "{}",
                    "addition of `output_wasm_path` field to BuildInfo (NEP330 1.3.0 extension)"
                        .yellow()
                );
                println!("{}", "Reproducible build verification of product contracts, deployed from such factories, won't be successful.".yellow());
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
                println!(
                    "{} {}",
                    "See examples at: ".yellow(),
                    format!(
                        "https://docs.rs/cargo-near-build/{}/cargo_near_build/extended/index.html",
                        CARGO_NEAR_BUILD_MIN
                    )
                    .cyan()
                );
                println!();
                std::thread::sleep(Duration::new(5, 0));
            }
            (false, true) => {
                println!(
                    "{}: {}",
                    "WARNING".red(),
                    "incompatible versions of `cargo-near(docker image)` and `cargo-near-build(build-dependencies)` have been detected: ".yellow()
                );
                println!(
                    "{}",
                    "addition of `output_wasm_path` field to BuildInfo (NEP330 1.3.0 extension)"
                        .yellow()
                );
                println!("{}", "Reproducible build verification of product contracts, deployed from such factories, won't be successful.".yellow());
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
                std::thread::sleep(Duration::new(5, 0));
            }
        }
    }
}

pub fn suggest_cargo_near_build_checks(
    crate_metadata: &CrateMetadata,
    reproducible_build: &ReproducibleBuild,
    near_sdk_support: NearSdkFeatureSupport,
) {
    let cargo_near = find_cargo_near_in_docker_img_tag(reproducible_build);
    let build_script = find_cargo_near_build_build_dep(crate_metadata);

    if let Some(cargo_near) = cargo_near.clone() {
        output_wasm_path::cargo_near_version_check(cargo_near);
    }

    if near_sdk_support.output_wasm_path {
        if let (Some(cargo_near), Some(build_script)) = (cargo_near, build_script) {
            output_wasm_path::with_buildscript_versions_check(cargo_near, build_script);
        }
    }
}

const PROD_IMAGE: &str = "sourcescan/cargo-near";
const DEV_IMAGE: &str = "dj8yfo/sourcescan";

fn find_cargo_near_in_docker_img_tag(
    reproducible_build: &ReproducibleBuild,
) -> Option<cargo_metadata::semver::Version> {
    let regex = regex::Regex::new(DOCKER_IMAGE_REGEX_PATTERN).expect("no error");
    let image_match = regex
        .captures(&reproducible_build.image)
        .and_then(|captures| captures.name("image"))?;

    if image_match.as_str() != PROD_IMAGE && image_match.as_str() != DEV_IMAGE {
        return None;
    }

    let tag_match = regex
        .captures(&reproducible_build.image)
        .and_then(|captures| captures.name("tag"))?;

    let regex2 = regex::Regex::new(SOURCE_SCAN_TAG_PATTERN).expect("no error");
    let cargo_near_version_match = regex2
        .captures(tag_match.as_str())
        .and_then(|captures| captures.get(1))?;

    cargo_metadata::semver::Version::from_str(cargo_near_version_match.as_str()).ok()
}

fn find_cargo_near_build_build_dep(
    crate_metadata: &CrateMetadata,
) -> Option<cargo_metadata::semver::Version> {
    match crate_metadata.find_direct_dependency("cargo_near_build") {
        Ok(packages) => {
            let maybe_package = packages
                .into_iter()
                .find(|(_package, dep_kinds)| {
                    dep_kinds
                        .iter()
                        .any(|dep_kind_info| dep_kind_info.kind == DependencyKind::Build)
                })
                .map(|(package, _dep_kinds)| package);

            maybe_package.map(|pkg| pkg.version.clone())
        }
        Err(err) => {
            // we cannot return this error, as this warning is only a recommendation,
            // and isn't a showstopper
            println!(
                "{}",
                "Encountered error when querying `cargo_near_build` dependency version".red()
            );
            println!("{}", err);
            None
        }
    }
}
