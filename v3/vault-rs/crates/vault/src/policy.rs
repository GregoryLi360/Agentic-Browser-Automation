//! Default policy — the impl of `vault_core::policy::TargetPolicy`.
//!
//! [`Allowlist`] is a file-backed allowlist + surface binding gate.

pub mod allowlist;

pub use allowlist::Allowlist;
