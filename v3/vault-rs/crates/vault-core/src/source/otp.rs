//! OTP layer: turn a stored [`Totp`] seed into a current code. Swappable behind the
//! trait (in-process generation, or a backend's own generator).

pub mod error;

pub use error::OtpError;

use crate::model::{Secret, Totp};

pub trait OtpGenerator {
    /// The current code for `totp`, returned as a [`Secret`] (short-lived but sensitive).
    fn generate(&self, totp: &Totp) -> Result<Secret, OtpError>;
}
