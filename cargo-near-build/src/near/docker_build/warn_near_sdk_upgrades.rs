use colored::Colorize;

use crate::types::cargo::metadata::CrateMetadata;
use std::time::Duration;

const MIN_CUTOFF: semver::Version = semver::Version::new(5, 2, 0);

const NEP330_1_3_0_CUTOFF: semver::Version = semver::Version::new(5, 11, 0);

pub fn suggest(crate_metadata: &CrateMetadata) {
    match crate_metadata.find_direct_dependency("near_sdk") {
        Ok(packages) => {
            for package in packages {
                if package.version < MIN_CUTOFF {
                    println!(
                        "{}: {}",
                        "WARNING".truecolor(220, 77, 1),
                        "a `near-sdk` package version has been detected, which doesn't support reproducible builds at all!".red()
                    );
                    println!(
                        "{} < {}",
                        format!("{}", package.version).red(),
                        format!("{}", MIN_CUTOFF).cyan()
                    );
                    println!(
                        "{} {}",
                        "An upgrade recommended up to".red(),
                        format!("{}", MIN_CUTOFF).cyan()
                    );
                    std::thread::sleep(Duration::new(10, 0));
                    println!();
                } else if package.version < NEP330_1_3_0_CUTOFF {
                    println!(
                        "{}: {}",
                        "WARNING".red(),
                        "a `near-sdk` package version has been detected, which doesn't support latest reproducible builds NEP330 1.3.0 extension".yellow() // deep orange
                    );
                    println!(
                        "{} < {}",
                        format!("{}", package.version).yellow(),
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
