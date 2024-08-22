use std::marker::PhantomData;

use camino::Utf8PathBuf;
use colored::{ColoredString, Colorize};
use sha2::{Digest, Sha256};

use crate::{
    cargo_native::{ArtifactType, Wasm},
    types::color_preference::ColorPreference,
    ManifestPath,
};

pub mod version_mismatch;

pub struct CompilationArtifact<T: ArtifactType = Wasm> {
    pub path: Utf8PathBuf,
    pub fresh: bool,
    pub from_docker: bool,
    // TODO: make pub(crate)
    pub builder_version_mismatch: version_mismatch::VersionMismatch,
    // TODO: make pub(crate)
    pub artifact_type: PhantomData<T>,
}

impl crate::BuildArtifact {
    pub fn compute_hash(&self) -> eyre::Result<SHA256Checksum> {
        let mut hasher = Sha256::new();
        hasher.update(std::fs::read(&self.path)?);
        let hash = hasher.finalize();
        let hash: &[u8] = hash.as_ref();
        Ok(SHA256Checksum {
            hash: hash.to_vec(),
        })
    }
}

pub struct SHA256Checksum {
    pub hash: Vec<u8>,
}

impl SHA256Checksum {
    pub fn to_hex_string(&self) -> String {
        hex::encode(&self.hash)
    }

    pub fn to_base58_string(&self) -> String {
        bs58::encode(&self.hash).into_string()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Opts {
    /// disable implicit `--locked` flag for all `cargo` commands, enabled by default
    pub no_locked: bool,
    /// Build contract in debug mode, without optimizations and bigger is size
    pub no_release: bool,
    /// Do not generate ABI for the contract
    pub no_abi: bool,
    /// Do not embed the ABI in the contract binary
    pub no_embed_abi: bool,
    /// Do not include rustdocs in the embedded ABI
    pub no_doc: bool,
    /// Copy final artifacts to this directory
    pub out_dir: Option<camino::Utf8PathBuf>,
    /// Path to the `Cargo.toml` of the contract to build
    pub manifest_path: Option<camino::Utf8PathBuf>,
    /// Set compile-time feature flags.
    pub features: Option<String>,
    /// Disables default feature flags.
    pub no_default_features: bool,
    /// Coloring: auto, always, never
    pub color: Option<ColorPreference>,
    /// description of cli command, where [crate::BuildOpts] are being used from, either real
    /// or emulated
    pub cli_description: CliDescription,
}

#[derive(Debug, Clone)]
pub struct CliDescription {
    /// binary name for builder field in ABI
    ///
    /// this is `"cargo-near"` in [std::default::Default] implementation
    pub cli_name_abi: String,
    /// cli command prefix for export of [crate::env_keys::nep330::BUILD_COMMAND] variable
    /// when used as lib method
    ///
    /// this is `vec!["cargo", "near", "build"]` in [std::default::Default] implementation
    pub cli_command_prefix: Vec<String>,
}

impl Default for CliDescription {
    fn default() -> Self {
        Self {
            cli_name_abi: "cargo-near".into(),
            cli_command_prefix: vec!["cargo".into(), "near".into(), "build".into()],
        }
    }
}

impl Opts {
    /// this is just 1-to-1 mapping of each struct's field to a cli flag
    /// in order of fields, as specified in struct's definition.
    /// `Default` implementation corresponds to plain `cargo near build` command without any args
    pub(crate) fn get_cli_build_command(&self) -> Vec<String> {
        let cargo_args = self.cli_description.cli_command_prefix.clone();
        let mut cargo_args: Vec<&str> = cargo_args.iter().map(|ele| ele.as_str()).collect();
        if self.no_locked {
            cargo_args.push("--no-locked");
        }
        // `no_docker` field isn't present
        if self.no_release {
            cargo_args.push("--no-release");
        }
        if self.no_abi {
            cargo_args.push("--no-abi");
        }
        if self.no_embed_abi {
            cargo_args.push("--no-embed-abi");
        }
        if self.no_doc {
            cargo_args.push("--no-doc");
        }
        if let Some(ref out_dir) = self.out_dir {
            cargo_args.extend_from_slice(&["--out-dir", out_dir.as_str()]);
        }
        if let Some(ref manifest_path) = self.manifest_path {
            cargo_args.extend_from_slice(&["--manifest-path", manifest_path.as_str()]);
        }
        if let Some(ref features) = self.features {
            cargo_args.extend(&["--features", features]);
        }
        if self.no_default_features {
            cargo_args.push("--no-default-features");
        }
        let color;
        if let Some(ref color_arg) = self.color {
            color = color_arg.to_string();
            cargo_args.extend(&["--color", &color]);
        }
        cargo_args
            .into_iter()
            .map(|el| el.to_string())
            .collect::<Vec<_>>()
    }
}

impl Opts {
    pub fn contract_path(&self) -> eyre::Result<camino::Utf8PathBuf> {
        let contract_path: camino::Utf8PathBuf = if let Some(manifest_path) = &self.manifest_path {
            let manifest_path = ManifestPath::try_from(manifest_path.clone())?;
            manifest_path.directory()?.to_path_buf()
        } else {
            camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
                .map_err(|err| eyre::eyre!("Failed to convert path {}", err.to_string_lossy()))?
        };
        Ok(contract_path)
    }

    const BUILD_COMMAND_CLI_CONFIG_ERR: &'static str =  "cannot be used, when `container_build_command` is configured from `[package.metadata.near.reproducible_build]` in Cargo.toml";

    pub fn get_cli_build_command_in_docker(
        &self,
        manifest_command: Option<Vec<String>>,
    ) -> eyre::Result<String> {
        if let Some(cargo_cmd) = manifest_command {
            // NOTE: `--no-locked` is allowed for docker builds
            // if self.no_locked {
            //     no-op
            // }
            if self.no_release {
                return Err(eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-release",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_abi {
                return Err(eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-abi",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_embed_abi {
                return Err(eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-embed-abi",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_doc {
                return Err(eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-doc",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.features.is_some() {
                return Err(eyre::eyre!(format!(
                    "`{}` {}",
                    "--features",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            if self.no_default_features {
                return Err(eyre::eyre!(format!(
                    "`{}` {}",
                    "--no-default-features",
                    Self::BUILD_COMMAND_CLI_CONFIG_ERR
                )));
            }
            return Ok(cargo_cmd.join(" "));
        }
        println!(
            "{}",
            "configuring `container_build_command` from cli args, passed to current command".cyan()
        );
        let mut cargo_args = vec![];
        // NOTE: not passing through `no_locked` to cmd in container,
        // an invisible Cargo.lock was generated by implicit `cargo metadata` anyway
        // if self.no_locked {
        //     no-op
        // }
        if self.no_release {
            cargo_args.push("--no-release");
        }
        if self.no_abi {
            cargo_args.push("--no-abi");
        }
        if self.no_embed_abi {
            cargo_args.push("--no-embed-abi");
        }
        if self.no_doc {
            cargo_args.push("--no-doc");
        }
        if let Some(ref features) = self.features {
            cargo_args.extend(&["--features", features]);
        }
        if self.no_default_features {
            cargo_args.push("--no-default-features");
        }

        let color;
        if let Some(ref color_arg) = self.color {
            color = color_arg.to_string();
            cargo_args.extend(&["--color", &color]);
        }
        // TODO: use cli_command_prefix
        //         let cargo_args = self.cli_description.cli_command_prefix.clone();
        let mut cargo_cmd_list = vec!["cargo", "near", "build"];
        cargo_cmd_list.extend(&cargo_args);
        let cargo_cmd = cargo_cmd_list.join(" ");
        Ok(cargo_cmd)
    }
}

#[derive(Default)]
pub struct ArtifactMessages<'a> {
    messages: Vec<(&'a str, ColoredString)>,
}

impl<'a> ArtifactMessages<'a> {
    pub fn push_binary(&mut self, artifact: &CompilationArtifact) -> eyre::Result<()> {
        self.messages
            .push(("Binary", artifact.path.to_string().bright_yellow().bold()));
        let checksum = artifact.compute_hash()?;
        self.messages.push((
            "SHA-256 checksum hex ",
            checksum.to_hex_string().green().dimmed(),
        ));
        self.messages.push((
            "SHA-256 checksum bs58",
            checksum.to_base58_string().green().dimmed(),
        ));
        Ok(())
    }
    pub fn push_free(&mut self, msg: (&'a str, ColoredString)) {
        self.messages.push(msg);
    }
    pub fn pretty_print(self) {
        let max_width = self.messages.iter().map(|(h, _)| h.len()).max().unwrap();
        for (header, message) in self.messages {
            eprintln!("     - {:>width$}: {}", header, message, width = max_width);
        }
    }
}
