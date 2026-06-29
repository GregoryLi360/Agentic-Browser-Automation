use thiserror::Error;

/// Failures of a password-manager backend. Scoped to the backend's one job —
/// reachability and session state. Matching, policy, and field-level concerns belong to
/// other layers.
#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("vault is locked")]
    Locked,
    #[error("not logged in to the vault")]
    LoggedOut,
    #[error("password manager unreachable: {0}")]
    Unreachable(String),
    #[error("could not unlock the vault: {0}")]
    Unlock(String),
    #[error("password manager backend error: {0}")]
    Backend(String),
}
