//! The authentication surface — what the agent is signing into (web page, native app),
//! and the [`Challenge`] iterator that drives the runtime.
//!
//! The surface has two jobs: **detect** the next authentication challenge
//! ([`next_challenge`](Surface::next_challenge)) and **apply** values to it
//! ([`fill`](Surface::fill) / [`submit`](Surface::submit)). *How* (DOM via JS, an
//! accessibility tree, a TUI) is the implementation's business — never in the contract.
//!
//! The broker calls `next_challenge` in a loop: satisfy the challenge, re-detect, repeat
//! until `None` (authenticated). That loop is the whole point — a real login can demand
//! an arbitrarily long sequence (identifier → password → TOTP → WebAuthn → consent → ...),
//! so the runtime is an iterator, not a fixed set of actions.

use thiserror::Error;

use crate::model::Target;

pub trait Surface {
    /// The live surface's identity, for target binding.
    fn target(&self) -> Result<Target, SurfaceError>;

    /// The next unsatisfied authentication challenge the surface presents, or `None` when
    /// there is nothing left to satisfy (assume authenticated). Re-evaluated each loop.
    fn next_challenge(&self) -> Result<Option<Challenge>, SurfaceError>;

    /// Put `value` in the field for `field` if the surface currently has it; `Ok(false)`
    /// if that field is absent. Does not submit.
    fn fill(&self, field: FieldKind, value: &str) -> Result<bool, SurfaceError>;

    /// Submit the current step, advancing the flow to whatever comes next.
    fn submit(&self) -> Result<(), SurfaceError>;
}

/// One step a surface demands — the authentication *method*, not a single field.
/// Satisfying it may fill several fields and submit; the next [`next_challenge`] re-detects
/// what comes after. A surface emits only what it can detect; richer detection (passkey
/// prompts, "Sign in with X" buttons, push waits) unlocks the later variants.
///
/// [`next_challenge`]: Surface::next_challenge
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Challenge {
    /// Username / identifier and/or password.
    Password,
    /// A one-time code field — TOTP (from a stored seed) or a delivered email/SMS code.
    /// The surface can't tell which; the broker resolves the source.
    OtpCode,
    /// A WebAuthn / passkey assertion.
    Passkey,
    /// Delegated "Sign in with <provider>" (OAuth/OIDC/SAML).
    Federated { provider: String },
    /// Out-of-band approval — push / number-match / QR — the broker can't satisfy itself.
    Approval { kind: String },
}

/// A low-level input field a surface fills. (Distinct from [`Challenge`]: a single
/// `Password` challenge may fill both `Username` and `Password` fields.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Username,
    Password,
    Otp,
}

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("surface unavailable: {0}")]
    Unavailable(String),
    #[error("surface action failed: {0}")]
    Failed(String),
}
