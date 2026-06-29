//! In-process TOTP generation (RFC 6238) via the totp-rs crate (RustCrypto-backed).
//! Generating in-process keeps the long-lived seed out of any argv.

use totp_rs::{Algorithm as TotpAlg, Secret as TotpSecret, TOTP};

use vault_core::model::{Algorithm, Secret, Totp};
use vault_core::source::otp::{OtpError, OtpGenerator};

pub struct TotpGenerator;

impl OtpGenerator for TotpGenerator {
    fn generate(&self, totp: &Totp) -> Result<Secret, OtpError> {
        let algorithm = match totp.algorithm {
            Algorithm::Sha1 => TotpAlg::SHA1,
            Algorithm::Sha256 => TotpAlg::SHA256,
            Algorithm::Sha512 => TotpAlg::SHA512,
        };
        let bytes = TotpSecret::Encoded(totp.secret.expose().to_string())
            .to_bytes()
            .map_err(|e| OtpError::BadSecret(format!("{e:?}")))?;
        let digits = totp.digits as usize;
        let period = totp.period as u64;
        let generator = TOTP::new(algorithm, digits, 1, period, bytes.clone())
            .unwrap_or_else(|_| TOTP::new_unchecked(algorithm, digits, 1, period, bytes));
        let code = generator.generate_current().map_err(|e| OtpError::Generate(e.to_string()))?;
        Ok(Secret::new(code))
    }
}
