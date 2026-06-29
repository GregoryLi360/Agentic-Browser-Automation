//! Sign-in *flows* — the methods that are **not** a credential fill. Two families the
//! `FormFiller` model can't express:
//!
//! - **delegated / redirect** — OAuth/OIDC/SAML "Sign in with <provider>", magic links:
//!   no secret goes into a field on the page; you *follow a flow* across origins (often
//!   recursively — the IdP login is itself a sign-in) to obtain a session.
//! - **out-of-band approval** — push / number-match (Duo, Okta), QR scan: nothing is
//!   typed; you trigger, then *wait on another channel*.
//!
//! The outcome is an authenticated **session**, not filled fields. A flow is multi-step,
//! may cross origins, and may wait — so it owns its own surface/transport and runs to an
//! [`FlowOutcome`] rather than handing the broker selectors. This is the second broker
//! capability beside [`FormFiller`](crate::fill::FormFiller): register a [`Flow`] and
//! OAuth / magic-link / push become new cells, no core change per method.
//!
//! Seam only — no impl ships here.

use thiserror::Error;

use crate::model::Target;

pub trait Flow {
    /// Which sign-in method this flow performs — for selection and reporting.
    fn kind(&self) -> FlowKind;

    /// Whether this flow can sign into `target` (e.g. its SSO button is present, or it is
    /// the magic-link provider for that domain).
    fn supports(&self, target: &Target) -> bool;

    /// Run the flow toward an authenticated session for `target`. Implementations drive
    /// their own navigation / waiting and may compose a fill internally (an IdP login);
    /// the broker only selects, invokes, and reports the outcome.
    fn run(&self, target: &Target) -> Result<FlowOutcome, FlowError>;
}

/// The family / method a [`Flow`] implements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowKind {
    /// OAuth / OIDC / SAML "Sign in with <provider>".
    Sso { provider: String },
    /// A link delivered out of band (email / SMS) that is navigated, not typed.
    MagicLink,
    /// Out-of-band approval on another device (push, number-match).
    PushApproval,
    /// A scanned code (QR) approved on another device.
    QrCode,
}

/// The result of running a [`Flow`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowOutcome {
    /// An authenticated session was established on the surface.
    SignedIn,
    /// The flow is waiting on an out-of-band action (a tap, a scan, an email click). The
    /// caller may poll/retry; `detail` describes what is pending.
    Pending { detail: String },
}

#[derive(Debug, Error)]
pub enum FlowError {
    #[error("no flow supports target '{0}'")]
    Unsupported(Target),
    #[error("flow step failed: {0}")]
    Failed(String),
    #[error("flow timed out waiting for out-of-band approval")]
    TimedOut,
}
