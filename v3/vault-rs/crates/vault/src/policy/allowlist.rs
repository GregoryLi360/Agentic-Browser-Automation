//! File-backed target allowlist. An absent file permits all targets; a present file
//! requires the requested target to match one of its entries (blank lines and `#`
//! comments ignored). Entries parse as targets, so `app:com.example` works alongside
//! bare hosts.

use std::fs;
use std::path::Path;

use vault_core::model::Target;
use vault_core::policy::{PolicyError, TargetPolicy};

pub struct Allowlist {
    path: String,
}

impl Allowlist {
    pub fn new(path: impl Into<String>) -> Self {
        Allowlist { path: path.into() }
    }
}

impl TargetPolicy for Allowlist {
    fn authorize(&self, requested: &Target) -> Result<(), PolicyError> {
        if !Path::new(&self.path).exists() {
            return Ok(());
        }
        let permitted = fs::read_to_string(&self.path)
            .map(|text| {
                text.lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .any(|entry| Target::parse(entry).matches(requested))
            })
            .unwrap_or(false);
        if permitted {
            Ok(())
        } else {
            Err(PolicyError::NotAllowed(requested.clone()))
        }
    }

    fn verify(&self, observed: &Target, requested: &Target) -> Result<(), PolicyError> {
        if observed.matches(requested) {
            Ok(())
        } else {
            Err(PolicyError::Mismatch { observed: observed.clone(), requested: requested.clone() })
        }
    }
}
