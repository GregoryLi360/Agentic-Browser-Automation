//! WebAuthn (FIDO2) assertion crypto — the part that signs the relying party's challenge
//! with a stored ES256 (P-256) passkey, *in Rust*, so the private key never enters the
//! page. Produces the three byte fields `navigator.credentials.get()` must resolve with.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use p256::pkcs8::DecodePrivateKey;
use sha2::{Digest, Sha256};

/// The signed pieces of a WebAuthn assertion (raw bytes, pre-encoding).
pub struct Assertion {
    pub client_data_json: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Build and sign an ES256 assertion.
///
/// - `origin`     the page's real origin, e.g. `https://github.com`
/// - `rp_id`      the relying-party id, e.g. `github.com`
/// - `challenge`  the RP's raw challenge bytes
/// - `sign_count` authenticator counter (0 is acceptable for a software authenticator)
/// - `pkcs8_der`  the P-256 private key in PKCS#8 DER
pub fn assert_es256(
    origin: &str,
    rp_id: &str,
    challenge: &[u8],
    sign_count: u32,
    pkcs8_der: &[u8],
) -> Result<Assertion, AssertError> {
    let client_data_json = format!(
        r#"{{"type":"webauthn.get","challenge":"{}","origin":"{}","crossOrigin":false}}"#,
        URL_SAFE_NO_PAD.encode(challenge),
        origin
    )
    .into_bytes();

    // authenticatorData = SHA-256(rpId) ‖ flags(UP|UV) ‖ signCount(be32)
    let mut authenticator_data = Vec::with_capacity(37);
    authenticator_data.extend_from_slice(&Sha256::digest(rp_id.as_bytes()));
    authenticator_data.push(0x05);
    authenticator_data.extend_from_slice(&sign_count.to_be_bytes());

    // signature = ECDSA-P256-SHA256( authenticatorData ‖ SHA-256(clientDataJSON) ), DER
    let key = SigningKey::from_pkcs8_der(pkcs8_der).map_err(|e| AssertError::Key(e.to_string()))?;
    let mut signed = authenticator_data.clone();
    signed.extend_from_slice(&Sha256::digest(&client_data_json));
    let signature: Signature = key.sign(&signed);

    Ok(Assertion {
        client_data_json,
        authenticator_data,
        signature: signature.to_der().as_bytes().to_vec(),
    })
}

#[derive(Debug)]
pub enum AssertError {
    Key(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdsa::{signature::Verifier, VerifyingKey};
    use p256::pkcs8::EncodePrivateKey;

    fn test_key() -> SigningKey {
        // A fixed, valid P-256 scalar — deterministic test, no RNG needed.
        SigningKey::from_slice(&[0x11u8; 32]).unwrap()
    }

    #[test]
    fn assertion_is_well_formed_and_verifies() {
        let key = test_key();
        let pkcs8 = key.to_pkcs8_der().unwrap();
        let a = assert_es256(
            "https://example.com",
            "example.com",
            b"a-random-challenge",
            0,
            pkcs8.as_bytes(),
        )
        .unwrap();

        // authenticatorData layout: SHA-256(rpId) ‖ 0x05 ‖ be32(0)
        assert_eq!(a.authenticator_data.len(), 37);
        assert_eq!(&a.authenticator_data[..32], &Sha256::digest(b"example.com")[..]);
        assert_eq!(a.authenticator_data[32], 0x05);
        assert_eq!(&a.authenticator_data[33..], &[0, 0, 0, 0]);

        // clientDataJSON shape
        let cd = String::from_utf8(a.client_data_json.clone()).unwrap();
        assert!(cd.contains(r#""type":"webauthn.get""#));
        assert!(cd.contains(r#""origin":"https://example.com""#));
        assert!(cd.contains(&URL_SAFE_NO_PAD.encode(b"a-random-challenge")));

        // signature verifies over authenticatorData ‖ SHA-256(clientDataJSON)
        let mut signed = a.authenticator_data.clone();
        signed.extend_from_slice(&Sha256::digest(&a.client_data_json));
        let vk = VerifyingKey::from(&key);
        let sig = Signature::from_der(&a.signature).unwrap();
        assert!(vk.verify(&signed, &sig).is_ok());
    }

    #[test]
    fn tampered_signature_fails() {
        let key = test_key();
        let pkcs8 = key.to_pkcs8_der().unwrap();
        let a = assert_es256("https://x.com", "x.com", b"chal", 0, pkcs8.as_bytes()).unwrap();

        // verify against the WRONG message (flipped a challenge byte) must fail
        let other =
            assert_es256("https://x.com", "x.com", b"chaL", 0, pkcs8.as_bytes()).unwrap();
        let mut signed = other.authenticator_data.clone();
        signed.extend_from_slice(&Sha256::digest(&other.client_data_json));
        let vk = VerifyingKey::from(&key);
        let sig = Signature::from_der(&a.signature).unwrap();
        assert!(vk.verify(&signed, &sig).is_err());
    }

    #[test]
    fn rejects_bad_key() {
        assert!(assert_es256("https://x.com", "x.com", b"c", 0, b"not-a-key").is_err());
    }
}
