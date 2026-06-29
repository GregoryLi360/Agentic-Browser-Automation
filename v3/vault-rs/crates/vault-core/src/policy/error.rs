use thiserror::Error;

use crate::model::Target;

/// Authorization failures — the security boundary that defeats inject-to-evil.com.
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("target '{0}' is not in the allowlist")]
    NotAllowed(Target),
    #[error("surface '{observed}' does not match requested target '{requested}' — refusing to fill")]
    Mismatch { observed: Target, requested: Target },
}
