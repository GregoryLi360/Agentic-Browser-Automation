//! Site origins — the normalized host the broker binds credentials and pages to.

use std::fmt;

/// A normalized site origin: a lowercased host. Build from a bare host or a full URL
/// via [`parse`](Origin::parse).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Origin(String);

impl Origin {
    /// Lowercase a bare host, or extract and lowercase the host from a URL.
    pub fn parse(input: &str) -> Self {
        let host = url::Url::parse(input).ok().and_then(|u| u.host_str().map(str::to_string));
        Origin(host.unwrap_or_else(|| input.to_string()).to_lowercase())
    }

    pub fn host(&self) -> &str {
        &self.0
    }

    /// True if these are the same host, or one is a subdomain of the other (covers
    /// `www` vs apex, and a parent-domain login filling a subdomain).
    pub fn matches(&self, other: &Origin) -> bool {
        let (a, b) = (self.host(), other.host());
        !a.is_empty()
            && !b.is_empty()
            && (a == b || a.ends_with(&format!(".{b}")) || b.ends_with(&format!(".{a}")))
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
