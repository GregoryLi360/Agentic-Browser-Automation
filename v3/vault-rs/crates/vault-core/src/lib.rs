//! `vault-core` — the interface crate for an agent-facing credential broker. Domain model
//! + the provider trait *slots* a broker composes. No concrete backends live here.
//!
//! - `model`  — shared vocabulary: `Item`, `Credential`, `Secret`, `Origin`, `Target`.
//! - `source` — `PasswordManager`, `OtpGenerator`, `VerificationSource` (+ the `Cached`
//!   decorator), re-exported so callers say `source::PasswordManager` (no stutter).
//! - `surface`— `Surface`: detects the `Challenge` a login presents and applies values.
//! - `policy` — `TargetPolicy`: authorize + bind targets.
//! - `flow`   — `Flow`: federated (OAuth/SSO, magic-link) sign-in.
//! - `passkey`— `PasskeyAuthenticator`: the WebAuthn assertion ceremony.

pub mod flow;
pub mod model;
pub mod passkey;
pub mod policy;
pub mod source;
pub mod surface;
