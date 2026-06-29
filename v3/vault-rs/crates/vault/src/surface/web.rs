//! `WebSurface` — the web impl of `vault_core::surface::Surface`. Detects the current
//! challenge with a [`Detector`] (DOM heuristic), fills via a [`PageDriver`], and submits
//! by requesting the tagged field's form. The detector tags chosen fields with
//! `data-vault-*`, which both `fill` (selectors) and `submit` (the field's form) reuse.

use std::cell::RefCell;

use vault_core::model::Target;
use vault_core::surface::{Challenge, FieldKind, Surface, SurfaceError};

use super::page::{DetectedFields, Detector, PageDriver};

/// Submit the form containing whichever login field the detector tagged.
const JS_SUBMIT: &str = r#"(() => {
  const e = document.querySelector('[data-vault-pass],[data-vault-user],[data-vault-otp]');
  if (e && e.form) { e.form.requestSubmit ? e.form.requestSubmit() : e.form.submit(); return true; }
  return false;
})()"#;

pub struct WebSurface<D: PageDriver, F: Detector> {
    driver: D,
    detector: F,
    detected: RefCell<Option<DetectedFields>>,
}

impl<D: PageDriver, F: Detector> WebSurface<D, F> {
    pub fn new(driver: D, detector: F) -> Self {
        WebSurface { driver, detector, detected: RefCell::new(None) }
    }
}

impl<D: PageDriver, F: Detector> Surface for WebSurface<D, F> {
    fn target(&self) -> Result<Target, SurfaceError> {
        let origin = self.driver.origin().map_err(|e| SurfaceError::Unavailable(e.to_string()))?;
        Ok(Target::Web(origin))
    }

    fn next_challenge(&self) -> Result<Option<Challenge>, SurfaceError> {
        let detected =
            self.detector.detect(&self.driver).map_err(|e| SurfaceError::Failed(e.to_string()))?;
        // Heuristic mapping. Passkey / federated / approval detection is the open frontier
        // and is not emitted here yet — drive those explicitly via `AuthOptions::force`.
        let challenge = if detected.password.is_some() || detected.username.is_some() {
            Some(Challenge::Password)
        } else if detected.otp.is_some() {
            Some(Challenge::OtpCode)
        } else {
            None
        };
        *self.detected.borrow_mut() = Some(detected);
        Ok(challenge)
    }

    fn fill(&self, field: FieldKind, value: &str) -> Result<bool, SurfaceError> {
        let detected = self.detected.borrow();
        let detected = detected
            .as_ref()
            .ok_or_else(|| SurfaceError::Failed("call next_challenge before fill".into()))?;
        let selector = match field {
            FieldKind::Username => detected.username.as_deref(),
            FieldKind::Password => detected.password.as_deref(),
            FieldKind::Otp => detected.otp.as_deref(),
        };
        match selector {
            Some(sel) => {
                self.driver
                    .fill(sel, value, false)
                    .map_err(|e| SurfaceError::Failed(e.to_string()))?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn submit(&self) -> Result<(), SurfaceError> {
        self.driver.eval(JS_SUBMIT).map_err(|e| SurfaceError::Failed(e.to_string()))?;
        Ok(())
    }
}
