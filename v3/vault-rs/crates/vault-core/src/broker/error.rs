use thiserror::Error;

use crate::flow::FlowError;
use crate::model::Target;
use crate::passkey::PasskeyError;
use crate::policy::PolicyError;
use crate::source::{ManagerError, OtpError, VerifyError};
use crate::surface::{Challenge, SurfaceError};

/// Everything that can go wrong in the authentication runtime. The broker's own
/// resolution failures, plus the layer errors aggregated via `#[from]`.
#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("no login found for '{0}'")]
    NotFound(Target),
    #[error("multiple logins match '{target}': {candidates}")]
    Ambiguous { target: Target, candidates: String },
    #[error("login '{item}' has no {kind} credential")]
    MissingCredential { item: String, kind: &'static str },
    #[error("no fillable login fields on the surface")]
    NoLoginFields,
    #[error("no OTP field on the surface")]
    NoOtpField,
    #[error("no registered flow can satisfy {0:?}")]
    NoFlow(Challenge),
    #[error("no passkey authenticator is registered")]
    NoPasskeyAuthenticator,
    #[error("no verification source is registered for a delivered code")]
    NoVerificationSource,
    #[error("stuck on {0:?} — the surface did not advance")]
    Stuck(Challenge),
    #[error("authentication did not complete within the step limit")]
    TooManySteps,

    #[error(transparent)]
    Policy(#[from] PolicyError),
    #[error(transparent)]
    Manager(#[from] ManagerError),
    #[error(transparent)]
    Surface(#[from] SurfaceError),
    #[error(transparent)]
    Otp(#[from] OtpError),
    #[error(transparent)]
    Verify(#[from] VerifyError),
    #[error(transparent)]
    Flow(#[from] FlowError),
    #[error(transparent)]
    Passkey(#[from] PasskeyError),
}
