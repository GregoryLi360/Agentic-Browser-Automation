//! What a credential is filled into — the identity the broker binds to. Web today;
//! native-app surfaces (bundle / package id) slot in without touching the fill path.

use std::fmt;

use crate::model::origin::Origin;

/// The surface a login belongs to. Generalizes a bare web [`Origin`] so the same broker,
/// policy, and matching logic cover a native app identified by bundle/package id.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    /// A web origin (host).
    Web(Origin),
    /// A native app, by bundle id / package name (e.g. `com.example.app`).
    App(String),
}

impl Target {
    /// Parse a caller-supplied target. An `app:<id>` prefix → [`Target::App`]; anything
    /// else (a bare host or a URL) → [`Target::Web`].
    pub fn parse(input: &str) -> Self {
        match input.strip_prefix("app:") {
            Some(id) => Target::App(id.trim().to_lowercase()),
            None => Target::Web(Origin::parse(input)),
        }
    }

    /// True if these identify the same surface — web hosts by [`Origin::matches`]
    /// (host / subdomain), apps by exact id. Never matches across kinds.
    pub fn matches(&self, other: &Target) -> bool {
        match (self, other) {
            (Target::Web(a), Target::Web(b)) => a.matches(b),
            (Target::App(a), Target::App(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Web(origin) => write!(f, "{origin}"),
            Target::App(id) => write!(f, "app:{id}"),
        }
    }
}
