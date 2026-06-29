//! Heuristic field detector — a prioritized CSS-selector sweep run in the page. It tags
//! the chosen username/password/OTP elements with `data-vault-*` attributes (no secrets
//! in these evals) and reports a selector for each. Swappable for a stronger engine
//! (Buttercup Locust / Proton Fathom) later; username selector order ported from
//! agent-vault (src/injector.ts, MIT).

use serde_json::Value;

use super::page::{DetectError, DetectedFields, Detector, PageDriver};

/// Returns `{username: bool, password: bool}` and tags the chosen visible elements.
const JS_TAG_LOGIN: &str = r#"(() => {
  const us = ["input[autocomplete=\"username\"]","input[autocomplete=\"email\"]","input[type=\"email\"]","input[name*=\"user\" i]","input[name*=\"email\" i]","input[name*=\"login\" i]","input[id*=\"user\" i]","input[id*=\"email\" i]","input[id*=\"login\" i]","input[type=\"text\"]"];
  let u = null;
  for (const s of us) { const e = document.querySelector(s); if (e && e.offsetParent !== null) { u = e; break; } }
  const p = document.querySelector('input[type="password"]');
  if (u) u.setAttribute('data-vault-user', '1');
  if (p) p.setAttribute('data-vault-pass', '1');
  return { username: !!u, password: !!p };
})()"#;

/// Returns a boolean and tags the first visible OTP-looking field.
const JS_TAG_OTP: &str = r#"(() => {
  const ss = ["input[autocomplete=\"one-time-code\"]","input[name*=\"otp\" i]","input[id*=\"otp\" i]","input[name*=\"totp\" i]","input[name*=\"2fa\" i]","input[name*=\"code\" i]","input[id*=\"code\" i]","input[inputmode=\"numeric\"]","input[type=\"tel\"]"];
  for (const s of ss) { const e = document.querySelector(s); if (e && e.offsetParent !== null) { e.setAttribute('data-vault-otp', '1'); return true; } }
  return false;
})()"#;

pub struct HeuristicDetector;

impl Detector for HeuristicDetector {
    fn detect(&self, page: &dyn PageDriver) -> Result<DetectedFields, DetectError> {
        let login = page.eval(JS_TAG_LOGIN)?;
        let otp = page.eval(JS_TAG_OTP)?;
        let found = |key: &str| login.get(key) == Some(&Value::Bool(true));
        Ok(DetectedFields {
            username: found("username").then(|| "[data-vault-user]".to_string()),
            password: found("password").then(|| "[data-vault-pass]".to_string()),
            otp: (otp == Value::Bool(true)).then(|| "[data-vault-otp]".to_string()),
        })
    }
}
