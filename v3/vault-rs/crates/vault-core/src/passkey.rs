//! Passkey assertion — deliberately a **separate capability from filling**, because a
//! passkey is never typed into a field. A WebAuthn / FIDO2 login is a challenge–response
//! *ceremony* (`navigator.credentials.get()`): the relying party issues a challenge, an
//! authenticator signs it with the private key bound to the origin, and returns an
//! assertion. There is no string to insert.
//!
//! So passkeys do **not** go through [`FormFiller`](crate::fill::FormFiller). To use one,
//! the broker must act as a *virtual authenticator* holding the credential — on the web
//! that is the CDP `WebAuthn` domain or the WebDriver virtual-authenticator extension,
//! not the page-fill path. This trait is the seam for that capability; a surface that
//! cannot host a virtual authenticator simply provides no implementation.
//!
//! No impl ships yet — this exists so the architecture is honest about where passkeys go
//! (a parallel slot beside `FormFiller`), not shoehorned into the fill model.

use thiserror::Error;

use crate::model::{Passkey, Target};

/// Satisfies a WebAuthn assertion for a stored [`Passkey`] by standing up a virtual
/// authenticator on the surface and completing the relying party's pending ceremony.
pub trait PasskeyAuthenticator {
    /// Register `passkey` as a virtual-authenticator credential for `target` and satisfy
    /// the page's pending `navigator.credentials.get()`.
    fn assert(&self, target: &Target, passkey: &Passkey) -> Result<(), PasskeyError>;
}

#[derive(Debug, Error)]
pub enum PasskeyError {
    #[error("surface cannot host a virtual authenticator: {0}")]
    Unsupported(String),
    #[error("passkey assertion failed: {0}")]
    Failed(String),
}
