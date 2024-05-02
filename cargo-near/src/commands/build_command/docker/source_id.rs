use std::{cmp::Ordering, fmt::Formatter};

use url::Url;

use self::canonical_url::CanonicalUrl;

pub mod canonical_url;

/// A type that can be converted to a Url
pub trait IntoUrl {
    /// Performs the conversion
    fn into_url(self) -> color_eyre::eyre::Result<Url>;
}

impl<'a> IntoUrl for &'a str {
    fn into_url(self) -> color_eyre::eyre::Result<Url> {
        Url::parse(self).map_err(|s| color_eyre::eyre::eyre!("invalid url `{}`: {}", self, s))
    }
}

#[derive(Clone, Eq, Debug)]
pub struct SourceId {
    /// The source URL.
    url: Url,
    /// The canonical version of the above url. See [`CanonicalUrl`] to learn
    /// why it is needed and how it normalizes a URL.
    canonical_url: canonical_url::CanonicalUrl,
    /// The source kind.
    kind: SourceKind,
    /// For example, the exact Git revision of the specified branch for a Git Source.
    precise: Option<Precise>,
}

impl PartialEq for SourceId {
    fn eq(&self, other: &SourceId) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for SourceId {
    fn partial_cmp(&self, other: &SourceId) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Custom comparison defined as canonical URL equality for git sources and URL
// equality for other sources, ignoring the `precise` and `name` fields.
impl Ord for SourceId {
    fn cmp(&self, other: &SourceId) -> Ordering {
        // Sort first based on `kind`, deferring to the URL comparison below if
        // the kinds are equal.
        match self.kind.cmp(&other.kind) {
            Ordering::Equal => {}
            other => return other,
        }

        // If the `kind` and the `url` are equal, then for git sources we also
        // ensure that the canonical urls are equal.
        match (&self.kind, &other.kind) {
            (SourceKind::Git(_), SourceKind::Git(_)) => {
                self.canonical_url.cmp(&other.canonical_url)
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
enum Precise {
    GitUrlFragment(String),
}

impl std::fmt::Display for Precise {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Precise::GitUrlFragment(s) => s.fmt(f),
        }
    }
}

/// Information to find a specific commit in a Git repository.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GitReference {
    /// From a specific revision. Can be a commit hash (either short or full),
    /// or a named reference like `refs/pull/493/head`.
    Rev(String),
}

impl GitReference {
    pub fn from_query(
        query_pairs: impl Iterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
    ) -> Self {
        let mut reference = GitReference::Rev("WRONG_REV".to_string());
        for (k, v) in query_pairs {
            let v = v.as_ref();
            match k.as_ref() {
                "rev" => reference = GitReference::Rev(v.to_owned()),
                _ => {}
            }
        }
        reference
    }
    /// Returns a `Display`able view of this git reference, or None if using
    /// the head of the default branch
    pub fn pretty_ref(&self, url_encoded: bool) -> Option<PrettyRef<'_>> {
        match self {
            _ => Some(PrettyRef {
                inner: self,
                url_encoded,
            }),
        }
    }
}

/// The possible kinds of code source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceKind {
    /// A git repository.
    Git(GitReference),
}

impl SourceKind {
    pub fn protocol(&self) -> Option<&str> {
        match self {
            SourceKind::Git(_) => Some("git"),
        }
    }
}

/// Forwards to `Ord`
impl PartialOrd for SourceKind {
    fn partial_cmp(&self, other: &SourceKind) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceKind {
    fn cmp(&self, other: &SourceKind) -> Ordering {
        match (self, other) {
            (SourceKind::Git(a), SourceKind::Git(b)) => a.cmp(b),
        }
    }
}

impl SourceId {
    /// Creates a `SourceId` object from the kind and URL.
    ///
    /// The canonical url will be calculated, but the precise field will not
    fn new(kind: SourceKind, url: Url) -> color_eyre::eyre::Result<SourceId> {
        let source_id = SourceId {
            kind,
            canonical_url: CanonicalUrl::new(&url)?,
            url,
            precise: None,
        };
        Ok(source_id)
    }

    /// Parses a source URL and returns the corresponding ID.
    ///
    /// ## Example
    ///
    /// ```
    /// use cargo::core::SourceId;
    /// SourceId::from_url("git+https://github.com/alexcrichton/\
    ///                     libssh2-static-sys#80e71a3021618eb05\
    ///                     656c58fb7c5ef5f12bc747f");
    /// ```
    #[allow(unused)]
    pub fn from_url(string: &str) -> color_eyre::eyre::Result<SourceId> {
        let (kind, url) = string
            .split_once('+')
            .ok_or_else(|| color_eyre::eyre::eyre!("invalid source `{}`", string))?;

        match kind {
            "git" => {
                let mut url = url.into_url()?;
                let reference = GitReference::from_query(url.query_pairs());
                let precise = url.fragment().map(|s| s.to_owned());
                url.set_fragment(None);
                url.set_query(None);
                Ok(SourceId::for_git(&url, reference)?.with_git_precise(precise))
            }
            kind => Err(color_eyre::eyre::eyre!(
                "unsupported source protocol: {}",
                kind
            )),
        }
    }

    /// Creates a new `SourceId` from this source with the given `precise`.
    #[allow(unused)]
    pub fn with_git_precise(self, fragment: Option<String>) -> SourceId {
        SourceId {
            precise: fragment.map(|f| Precise::GitUrlFragment(f)),
            ..self
        }
    }

    /// A view of the [`SourceId`] that can be `Display`ed as a URL.
    pub fn as_url(&self) -> SourceIdAsUrl<'_> {
        SourceIdAsUrl {
            inner: self,
            encoded: false,
        }
    }

    /// Creates a `SourceId` from a Git reference.
    pub fn for_git(url: &Url, reference: GitReference) -> color_eyre::eyre::Result<SourceId> {
        SourceId::new(SourceKind::Git(reference), url.clone())
    }

    /// Gets this source URL.
    #[allow(unused)]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Gets the canonical URL of this source, used for internal comparison
    /// purposes.
    #[allow(unused)]
    pub fn canonical_url(&self) -> &CanonicalUrl {
        &self.canonical_url
    }

    #[allow(unused)]
    pub fn kind(&self) -> &SourceKind {
        &self.kind
    }
}

/// A `Display`able view into a `SourceId` that will write it as a url
pub struct SourceIdAsUrl<'a> {
    inner: &'a SourceId,
    encoded: bool,
}

impl<'a> std::fmt::Display for SourceIdAsUrl<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(protocol) = self.inner.kind.protocol() {
            write!(f, "{protocol}+")?;
        }
        write!(f, "{}", self.inner.url)?;
        let SourceId {
            kind: SourceKind::Git(ref reference),
            ref precise,
            ..
        } = *self.inner;

        if let Some(pretty) = reference.pretty_ref(self.encoded) {
            write!(f, "?{}", pretty)?;
        }
        if let Some(precise) = precise.as_ref() {
            write!(f, "#{}", precise)?;
        }

        Ok(())
    }
}

/// A git reference that can be `Display`ed
pub struct PrettyRef<'a> {
    inner: &'a GitReference,
    url_encoded: bool,
}

impl<'a> std::fmt::Display for PrettyRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &str;
        match self.inner {
            GitReference::Rev(s) => {
                write!(f, "rev=")?;
                value = s;
            }
        }
        if self.url_encoded {
            for value in url::form_urlencoded::byte_serialize(value.as_bytes()) {
                write!(f, "{value}")?;
            }
        } else {
            write!(f, "{value}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::build_command::docker::source_id::{GitReference, SourceKind};

    use super::SourceId;

    #[test]
    fn test_source_id_from_url() {
        let cargo_source_id = SourceId::from_url(
            "git+https://github.com/dj8yfo/sample_no_workspace.git?rev=10415b1359c74b0d5774ce08b114f2bd1a85445d"
        ).unwrap();

        let kind = cargo_source_id.kind();
        assert!(
            matches!(kind, SourceKind::Git(GitReference::Rev(rev)) if rev == "10415b1359c74b0d5774ce08b114f2bd1a85445d")
        );

        assert_eq!(
            "/dj8yfo/sample_no_workspace.git",
            cargo_source_id.url().path()
        );
    }
}
