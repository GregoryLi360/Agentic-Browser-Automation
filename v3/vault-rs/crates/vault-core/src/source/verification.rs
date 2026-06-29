//! Verification layer: read delivered out-of-band codes (email / SMS). Kept separate
//! from the password manager because these codes are *delivered, never stored* — every
//! standard (WebOTP, Apple one-time-code) treats them as get-only. The broker consults a
//! source when a page asks for a code the manager cannot hold.

pub mod error;

pub use error::VerifyError;

use crate::model::Secret;

pub trait VerificationSource {
    /// The most recent code delivered over `channel`, as a [`Secret`].
    fn latest_code(&self, channel: Channel) -> Result<Secret, VerifyError>;
}

/// Out-of-band delivery channel for a verification code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Email,
    Sms,
}
