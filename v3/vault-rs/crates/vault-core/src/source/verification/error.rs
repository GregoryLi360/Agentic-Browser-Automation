use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("no verification code available: {0}")]
    NotFound(String),
    #[error("verification source error: {0}")]
    Source(String),
}
