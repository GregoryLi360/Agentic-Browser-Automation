use thiserror::Error;

#[derive(Debug, Error)]
pub enum OtpError {
    #[error("invalid TOTP secret: {0}")]
    BadSecret(String),
    #[error("could not generate TOTP code: {0}")]
    Generate(String),
}
