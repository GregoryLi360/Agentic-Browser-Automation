//! Source layer: where a fill-value comes from. Three independent providers, each a
//! trait the broker depends on — one per kind of secret the page can ask for:
//!
//! - `password_manager` — stored credentials (username, password, TOTP seed).
//! - `otp`              — a one-time code *generated* from a stored TOTP seed.
//! - `verification`     — a one-time code *delivered* out of band (email / SMS).
//!
//! Grouped because the broker treats them uniformly: each answers "what value do I
//! fill here?" without knowing about the page, policy, or one another. Each provider's
//! trait and types are re-exported here, so callers say `source::PasswordManager`
//! rather than the stuttering `source::password_manager::PasswordManager`.
//!
//! These are *interfaces only* — concrete backends (Bitwarden, the in-process TOTP
//! generator, ...) live in downstream impl crates. `Cached` is the one exception: a
//! generic, backend-agnostic decorator, so it ships with the interface.

pub mod otp;
pub mod password_manager;
pub mod verification;

pub use otp::{OtpError, OtpGenerator};
pub use password_manager::{Cached, ManagerError, PasswordManager, Status};
pub use verification::{Channel, VerificationSource, VerifyError};
