use super::output::CompilationArtifact;
use colored::{ColoredString, Colorize};

#[derive(Default)]
pub struct ArtifactMessages<'a> {
    messages: Vec<(&'a str, ColoredString)>,
}

impl<'a> ArtifactMessages<'a> {
    pub fn push_binary(&mut self, artifact: &CompilationArtifact) -> eyre::Result<()> {
        self.messages
            .push(("Binary", artifact.path.to_string().bright_yellow().bold()));
        let size_bytes = std::fs::metadata(&artifact.path)?.len();
        self.messages
            .push(("Size", format_size(size_bytes).cyan().dimmed()));
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
    #[cfg(feature = "build_internal")]
    pub fn push_free(&mut self, msg: (&'a str, ColoredString)) {
        self.messages.push(msg);
    }
    pub fn pretty_print(self) {
        let max_width = self.messages.iter().map(|(h, _)| h.len()).max().unwrap();
        for (header, message) in self.messages {
            eprintln!("     - {header:>max_width$}: {message}");
        }
    }
}

/// Formats a byte count as a human-readable size plus the exact byte count,
/// e.g. `112.8 KB (112824 bytes)`.
fn format_size(bytes: u64) -> String {
    let unit = if bytes == 1 { "byte" } else { "bytes" };
    format!("{} ({bytes} {unit})", bytesize::ByteSize::b(bytes))
}

#[cfg(test)]
mod tests {
    use super::format_size;

    #[test]
    fn format_size_keeps_exact_bytes_and_human_units() {
        let formatted = format_size(112_824);
        // exact byte count is always preserved for precision
        assert!(
            formatted.ends_with("(112824 bytes)"),
            "missing raw byte count: {formatted}"
        );
        // and a human-readable unit is shown alongside it
        assert!(
            formatted.contains('B') && formatted.contains("112824 bytes"),
            "unexpected format: {formatted}"
        );
    }

    #[test]
    fn format_size_handles_small_values() {
        assert_eq!(format_size(0), "0 B (0 bytes)");
        // singular `byte` for a 1-byte artifact, not `1 bytes`
        assert_eq!(format_size(1), "1 B (1 byte)");
        assert_eq!(format_size(2), "2 B (2 bytes)");
    }
}
