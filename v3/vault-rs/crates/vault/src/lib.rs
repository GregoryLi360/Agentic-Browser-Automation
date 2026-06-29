//! `vault` — one concrete composition ("matrix cell") of the `vault_core` slots: a
//! default impl per interface, each behind a default-on feature. **Every module mirrors
//! the `vault_core` slot it implements**, so the interface is unambiguous; [`defaults`]
//! bundles the chosen impls + `default_broker`.
//!
//! | module    | implements (`vault_core`)        | default impl(s)                          |
//! |-----------|----------------------------------|------------------------------------------|
//! | [`source`]  | `source::PasswordManager`, `source::OtpGenerator` | `BitwardenCli`, `TotpGenerator` |
//! | [`surface`] | `surface::Surface`              | `WebSurface` (+ `surface::page` internals) |
//! | [`policy`]  | `policy::TargetPolicy`          | `Allowlist`                              |
//! | [`passkey`] | `passkey::PasskeyAuthenticator` | `PageInjectionPasskey`                   |
//! | [`defaults`]| — (the batteries bundle)        | re-exports + `default_broker`            |
//!
//! Swap any module's impl for another cell (1Password, an Appium `Surface`, a Fathom
//! detector, a CDP passkey authenticator, ...) without touching `vault_core`.

#[cfg(feature = "web")]
pub mod surface;
#[cfg(feature = "passkey")]
pub mod passkey;
#[cfg(feature = "allowlist")]
pub mod policy;
#[cfg(any(feature = "bitwarden", feature = "totp"))]
pub mod source;

pub mod defaults;

#[cfg(all(feature = "bitwarden", feature = "web", feature = "totp", feature = "allowlist"))]
pub use defaults::default_broker;
