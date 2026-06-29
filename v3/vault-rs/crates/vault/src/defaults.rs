//! The defaults `vault` provides — one impl per `vault_core` slot, plus the
//! batteries-included [`default_broker`] wiring them into the authentication runtime. This
//! is the one place that says "here is our default cell of the provider matrix":
//!
//! | slot (vault_core trait)         | default impl             | feature     |
//! |---------------------------------|--------------------------|-------------|
//! | `source::PasswordManager`       | [`BitwardenCli`]         | `bitwarden` |
//! | `source::OtpGenerator`          | [`TotpGenerator`]        | `totp`      |
//! | `surface::Surface`              | [`WebSurface`]           | `web`       |
//! | `policy::TargetPolicy`          | [`Allowlist`]            | `allowlist` |
//! | `passkey::PasskeyAuthenticator` | [`PageInjectionPasskey`] | `passkey`   |
//!
//! Not covered by a default: `source::VerificationSource`, `flow::Flow` — seams with no
//! impl yet (so delivered-code, OAuth/SSO, magic-link, and push challenges have no
//! satisfier in the default cell).

#[cfg(feature = "bitwarden")]
pub use crate::source::BitwardenCli;
#[cfg(feature = "totp")]
pub use crate::source::TotpGenerator;
#[cfg(feature = "web")]
pub use crate::surface::WebSurface;
#[cfg(feature = "allowlist")]
pub use crate::policy::Allowlist;
#[cfg(feature = "passkey")]
pub use crate::passkey::PageInjectionPasskey;

/// The batteries-included broker: Bitwarden × web surface (Playwright + heuristic) × TOTP
/// × allowlist, plus the page-injection passkey authenticator when the `passkey` feature
/// is on. `session` is the playwright-cli session name; the allowlist path comes from
/// `$VAULT_ALLOW` (default `vault.allow`).
///
/// Compiled only when all four required default-impl features are enabled; otherwise
/// compose your own with [`vault_core::broker::BrokerService::new`].
#[cfg(all(feature = "bitwarden", feature = "web", feature = "totp", feature = "allowlist"))]
pub fn default_broker(session: &str) -> vault_core::broker::BrokerService {
    use crate::surface::{HeuristicDetector, PlaywrightDriver};
    use vault_core::source::Cached;

    let allow = std::env::var("VAULT_ALLOW").unwrap_or_else(|_| "vault.allow".into());
    let broker = vault_core::broker::BrokerService::new(
        Cached::new(BitwardenCli::new()),
        WebSurface::new(PlaywrightDriver::new(session.to_string()), HeuristicDetector),
        TotpGenerator,
        Allowlist::new(allow),
    );
    #[cfg(feature = "passkey")]
    let broker = broker
        .with_passkey(PageInjectionPasskey::new(PlaywrightDriver::new(session.to_string())));
    broker
}
