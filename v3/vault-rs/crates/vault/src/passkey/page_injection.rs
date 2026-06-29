//! Page-injection passkey authenticator — the Bitwarden-extension approach. A script
//! overrides `navigator.credentials.get()` in the page, parks the relying party's
//! challenge on `window`, the broker signs it **in Rust** (the key never enters the
//! page), and the script resolves the RP's promise with the assertion. Browser-agnostic;
//! no CDP virtual authenticator needed.
//!
//! Status: the [`assertion`] crypto is unit-tested; the page bridge below is **not yet
//! tested against a live relying party.** Known caveats:
//! - the returned credential is duck-typed, not a real `PublicKeyCredential` instance, so
//!   an RP that checks `instanceof PublicKeyCredential` will reject it;
//! - the hook is installed when [`assert`](PageInjectionPasskey::assert) runs, so it must
//!   be called *before* the RP invokes `get()` (an init-script at document-start would be
//!   strictly better, if the driver exposes one);
//! - `Passkey::key` is assumed to be a PKCS#8 DER P-256 key, standard-base64 encoded.


use std::thread::sleep;
use std::time::Duration;

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::json;

use vault_core::model::{Passkey, Target};
use vault_core::passkey::{PasskeyAuthenticator, PasskeyError};

use super::assertion::assert_es256;
use crate::surface::page::PageDriver;

/// Overrides `navigator.credentials.get`, parks the request on `window.__vaultPasskeyReq`,
/// and resolves once the broker writes `window.__vaultPasskeyResp`.
const JS_INSTALL: &str = r#"(() => {
  if (window.__vaultPasskeyHooked) return true;
  window.__vaultPasskeyHooked = true;
  const b64url = (buf) => { const b = new Uint8Array(buf); let s = ''; for (const x of b) s += String.fromCharCode(x); return btoa(s).replace(/\+/g,'-').replace(/\//g,'_').replace(/=+$/,''); };
  const toBuf = (s) => { s = s.replace(/-/g,'+').replace(/_/g,'/'); const bin = atob(s); const u = new Uint8Array(bin.length); for (let i=0;i<bin.length;i++) u[i]=bin.charCodeAt(i); return u.buffer; };
  navigator.credentials.get = (opts) => {
    const pk = opts && opts.publicKey;
    if (!pk) return Promise.reject(new DOMException('no publicKey', 'NotAllowedError'));
    window.__vaultPasskeyResp = null;
    window.__vaultPasskeyReq = { challenge: b64url(pk.challenge), rpId: pk.rpId || location.hostname };
    return new Promise((resolve, reject) => {
      const t0 = Date.now();
      const timer = setInterval(() => {
        const r = window.__vaultPasskeyResp;
        if (r) {
          clearInterval(timer);
          resolve({
            id: r.id, type: 'public-key', rawId: toBuf(r.rawId),
            authenticatorAttachment: 'platform',
            response: {
              clientDataJSON: toBuf(r.clientDataJSON),
              authenticatorData: toBuf(r.authenticatorData),
              signature: toBuf(r.signature),
              userHandle: r.userHandle ? toBuf(r.userHandle) : null,
            },
            getClientExtensionResults: () => ({}),
          });
        } else if (Date.now() - t0 > 120000) { clearInterval(timer); reject(new DOMException('timeout', 'NotAllowedError')); }
      }, 50);
    });
  };
  return true;
})()"#;

#[derive(Deserialize)]
struct Request {
    challenge: String,
    #[serde(rename = "rpId")]
    rp_id: String,
}

pub struct PageInjectionPasskey<D: PageDriver> {
    driver: D,
    max_polls: u32,
}

impl<D: PageDriver> PageInjectionPasskey<D> {
    /// Polls up to ~120s (600 × 200ms) for the page to invoke `get()`.
    pub fn new(driver: D) -> Self {
        PageInjectionPasskey { driver, max_polls: 600 }
    }

    fn poll_request(&self) -> Result<Request, PasskeyError> {
        for _ in 0..self.max_polls {
            let value = self
                .driver
                .eval("window.__vaultPasskeyReq || null")
                .map_err(|e| PasskeyError::Failed(e.to_string()))?;
            if !value.is_null() {
                return serde_json::from_value(value)
                    .map_err(|e| PasskeyError::Failed(format!("bad passkey request: {e}")));
            }
            sleep(Duration::from_millis(200));
        }
        Err(PasskeyError::Failed("timed out waiting for navigator.credentials.get()".into()))
    }
}

impl<D: PageDriver> PasskeyAuthenticator for PageInjectionPasskey<D> {
    fn assert(&self, _target: &Target, passkey: &Passkey) -> Result<(), PasskeyError> {
        self.driver.eval(JS_INSTALL).map_err(|e| PasskeyError::Failed(e.to_string()))?;

        let request = self.poll_request()?;

        let host = self
            .driver
            .origin()
            .map_err(|e| PasskeyError::Failed(e.to_string()))?
            .host()
            .to_string();
        let origin = format!("https://{host}");

        let challenge = URL_SAFE_NO_PAD
            .decode(request.challenge.as_bytes())
            .map_err(|e| PasskeyError::Failed(format!("bad challenge: {e}")))?;
        let pkcs8 = STANDARD
            .decode(passkey.key.expose().as_bytes())
            .map_err(|e| PasskeyError::Failed(format!("bad key encoding: {e}")))?;

        let assertion = assert_es256(&origin, &request.rp_id, &challenge, 0, &pkcs8)
            .map_err(|e| PasskeyError::Failed(format!("sign: {e:?}")))?;

        let response = json!({
            "id": URL_SAFE_NO_PAD.encode(&passkey.credential_id),
            "rawId": STANDARD.encode(&passkey.credential_id),
            "clientDataJSON": STANDARD.encode(&assertion.client_data_json),
            "authenticatorData": STANDARD.encode(&assertion.authenticator_data),
            "signature": STANDARD.encode(&assertion.signature),
            "userHandle": STANDARD.encode(&passkey.user_handle),
        });
        self.driver
            .eval(&format!("window.__vaultPasskeyResp = {response}; true"))
            .map_err(|e| PasskeyError::Failed(e.to_string()))?;
        Ok(())
    }
}
