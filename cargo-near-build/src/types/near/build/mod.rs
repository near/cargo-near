use std::marker::PhantomData;

use camino::Utf8PathBuf;
use colored::{ColoredString, Colorize};
use sha2::{Digest, Sha256};

use crate::{
    cargo_native::{ArtifactType, Wasm},
    types::color_preference::ColorPreference,
};

pub mod version_mismatch;

pub struct CompilationArtifact<T: ArtifactType = Wasm> {
    pub path: Utf8PathBuf,
    pub fresh: bool,
    pub from_docker: bool,
    pub builder_version_mismatch: version_mismatch::VersionMismatch,
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
}

impl Opts {
    /// this is just 1-to-1 mapping of each struct's field to a cli flag
    /// in order of fields, as specified in struct's definition.
    /// `Default` implementation corresponds to plain `cargo near build` command without any args
    pub(crate) fn get_cli_build_command(&self) -> Vec<String> {
        let mut cargo_args = vec!["cargo", "near", "build"];
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
