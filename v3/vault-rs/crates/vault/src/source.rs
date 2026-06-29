//! Default credential sources — impls of the `vault_core::source` provider traits.
//!
//! - [`BitwardenCli`] implements `vault_core::source::PasswordManager` (the `bw` CLI).
//! - [`TotpGenerator`] implements `vault_core::source::OtpGenerator` (in-process RFC 6238).

#[cfg(feature = "bitwarden")]
pub mod bitwarden;
#[cfg(feature = "totp")]
pub mod totp;

#[cfg(feature = "bitwarden")]
pub use bitwarden::BitwardenCli;
#[cfg(feature = "totp")]
pub use totp::TotpGenerator;
