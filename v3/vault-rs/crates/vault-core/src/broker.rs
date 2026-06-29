//! Broker layer: the agent-facing surface, and the **challenge-iterator authentication
//! runtime**. `authenticate` loops over the surface's `next_challenge`, dispatching each
//! to the matching satisfier (credential source + surface fill, OTP, passkey, flow) until
//! the surface is satisfied — so password / OTP / passkey / federated sign-in are all
//! degenerate cases of one loop, and arbitrarily long sequences just work.
//!
//! Enforces target policy + secret hygiene. Owns no storage, no surface, no crypto; only
//! coordination. Returns only non-secret results.

pub mod error;
pub mod service;

pub use error::BrokerError;
pub use service::BrokerService;

use crate::model::{ItemSummary, Target};
use crate::source::Status;
use crate::surface::Challenge;

pub trait Broker {
    /// Run the authentication runtime against `target`: detect the next challenge, satisfy
    /// it, repeat until the surface reports none left (or an out-of-band step is reached).
    fn authenticate(&self, target: &Target, opts: AuthOptions) -> Result<AuthOutcome, BrokerError>;

    /// Available logins, secret-free.
    fn list(&self) -> Result<Vec<ItemSummary>, BrokerError>;

    /// Password-manager reachability / lock state.
    fn status(&self) -> Result<Status, BrokerError>;

    /// Ensure the password manager is unlocked.
    fn unlock(&self) -> Result<(), BrokerError>;
}

/// How the runtime should behave.
#[derive(Debug, Clone, Default)]
pub struct AuthOptions {
    /// Submit each step after filling it. Off = fill only (the caller submits).
    pub submit: bool,
    /// Skip the surface-target == requested-target binding check. Default enforces it.
    pub skip_page_check: bool,
    /// Force a specific first challenge instead of detecting it — lets a caller drive a
    /// modality the surface can't yet detect (e.g. passkey, a named SSO provider). The
    /// loop then continues by detection.
    pub force: Option<Challenge>,
}

/// The result of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthOutcome {
    /// The surface reports no further challenges.
    Authenticated { steps: Vec<Challenge> },
    /// Stopped at an out-of-band step the broker can't satisfy itself (push / QR / a flow
    /// awaiting the user). The caller waits and retries.
    Pending { waiting_on: Challenge, steps: Vec<Challenge> },
}
