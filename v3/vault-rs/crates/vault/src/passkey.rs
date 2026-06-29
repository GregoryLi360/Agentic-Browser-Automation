//! Default passkey authenticator — the impl of `vault_core::passkey::PasskeyAuthenticator`.
//!
//! [`PageInjectionPasskey`] overrides `navigator.credentials.get()` in the page and signs
//! the WebAuthn assertion in Rust (the [`assertion`] submodule), so the private key never
//! enters the page.

pub mod assertion;
mod page_injection;

pub use page_injection::PageInjectionPasskey;
