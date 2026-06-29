//! The credential model, taken from the cross-manager standards (FIDO Credential
//! Exchange Format, W3C Credential Management, Apple/Android credential providers):
//! an [`Item`] is one account that carries a set of typed [`Credential`] facets, so a
//! single login can hold a password, a TOTP seed, and a passkey at once.

use crate::model::origin::Origin;
use crate::model::secret::Secret;
use crate::model::target::Target;

/// A vault entry — one account login. (CXF `Item`.)
#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub urls: Vec<String>,
    pub credentials: Vec<Credential>,
}

/// One typed secret facet of an [`Item`]. Discriminated like the CXF `CredentialType`
/// and the W3C Credential `type`.
#[derive(Debug, Clone)]
pub enum Credential {
    /// Username + password. (CXF `basic-auth`.)
    BasicAuth { username: Option<String>, password: Secret },
    /// Time-based one-time-password seed. (CXF `totp`.)
    Totp(Totp),
    /// WebAuthn / FIDO2 passkey. (CXF `passkey`.) Not yet fillable.
    Passkey(Passkey),
}

/// The kind of a [`Credential`] without its secret — used to match detected page fields
/// to available facets and for secret-free listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKind {
    BasicAuth,
    Totp,
    Passkey,
}

/// A TOTP seed and its parameters. (CXF `totp`.)
#[derive(Debug, Clone)]
pub struct Totp {
    pub secret: Secret,
    pub period: u32,
    pub digits: u8,
    pub algorithm: Algorithm,
    pub issuer: Option<String>,
}

/// TOTP hash algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Sha1,
    Sha256,
    Sha512,
}

/// A passkey's stored material. (CXF `passkey`.)
#[derive(Debug, Clone)]
pub struct Passkey {
    pub credential_id: Vec<u8>,
    pub rp_id: String,
    pub user_handle: Vec<u8>,
    pub key: Secret,
}

/// Secret-free descriptor of an [`Item`] — what listing returns. (Cf. Apple
/// `ASCredentialIdentity`: an advertised identity carrying no secret material.)
#[derive(Debug, Clone)]
pub struct ItemSummary {
    pub name: String,
    pub urls: Vec<String>,
    pub kinds: Vec<CredentialKind>,
}

impl Credential {
    pub fn kind(&self) -> CredentialKind {
        match self {
            Credential::BasicAuth { .. } => CredentialKind::BasicAuth,
            Credential::Totp(_) => CredentialKind::Totp,
            Credential::Passkey(_) => CredentialKind::Passkey,
        }
    }
}

impl Item {
    /// The first credential of `kind`, if this item has one.
    pub fn credential(&self, kind: CredentialKind) -> Option<&Credential> {
        self.credentials.iter().find(|c| c.kind() == kind)
    }

    /// Secret-free summary for listings.
    pub fn summary(&self) -> ItemSummary {
        ItemSummary {
            name: self.name.clone(),
            urls: self.urls.clone(),
            kinds: self.credentials.iter().map(Credential::kind).collect(),
        }
    }

    /// True if this item logs into `target` — a web host (either direction, or by name
    /// when the item has no URLs) or a native app id (substring of a URL or the name).
    pub fn matches(&self, target: &Target) -> bool {
        match target {
            Target::Web(origin) => self.matches_origin(origin),
            Target::App(id) => {
                let id = id.to_lowercase();
                !id.is_empty()
                    && (self.name.to_lowercase().contains(&id)
                        || self.urls.iter().any(|u| u.to_lowercase().contains(&id)))
            }
        }
    }

    fn matches_origin(&self, origin: &Origin) -> bool {
        if self.urls.is_empty() {
            let name = self.name.to_lowercase();
            return !name.is_empty()
                && (origin.host().contains(&name) || name.contains(origin.host()));
        }
        self.urls
            .iter()
            .any(|u| Origin::parse(u).matches(origin) || u.to_lowercase().contains(origin.host()))
    }
}

impl Totp {
    /// Parse a stored TOTP field — a raw base32 secret or an `otpauth://` URI (the
    /// cross-manager Key-Uri-Format) — into structured parameters.
    pub fn from_field(field: &str) -> Totp {
        let field = field.trim();
        if field.to_lowercase().starts_with("otpauth://") {
            if let Ok(url) = url::Url::parse(field) {
                let mut totp = Totp {
                    secret: Secret::new(String::new()),
                    period: 30,
                    digits: 6,
                    algorithm: Algorithm::Sha1,
                    issuer: None,
                };
                for (key, value) in url.query_pairs() {
                    match key.as_ref() {
                        "secret" => totp.secret = Secret::new(value.into_owned()),
                        "algorithm" => {
                            totp.algorithm = match value.to_uppercase().as_str() {
                                "SHA256" => Algorithm::Sha256,
                                "SHA512" => Algorithm::Sha512,
                                _ => Algorithm::Sha1,
                            }
                        }
                        "digits" => totp.digits = value.parse().unwrap_or(6),
                        "period" => totp.period = value.parse().unwrap_or(30),
                        "issuer" => totp.issuer = Some(value.into_owned()),
                        _ => {}
                    }
                }
                return totp;
            }
        }
        Totp {
            secret: Secret::new(field.replace(' ', "")),
            period: 30,
            digits: 6,
            algorithm: Algorithm::Sha1,
            issuer: None,
        }
    }
}
