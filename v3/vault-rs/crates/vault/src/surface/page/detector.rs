//! Field detector: locates fillable fields on the current page and reports a selector
//! for each. The detection engine is swappable behind this trait — our heuristic,
//! Buttercup Locust, or the Proton Fathom model — independent of the page driver.

use thiserror::Error;

use super::driver::{DriverError, PageDriver};

pub trait Detector {
    /// Inspect the page (via `page`) and report the fields it found.
    fn detect(&self, page: &dyn PageDriver) -> Result<DetectedFields, DetectError>;
}

/// CSS selectors for the fields found on the page, by role. `None` = not present.
#[derive(Debug, Default, Clone)]
pub struct DetectedFields {
    pub username: Option<String>,
    pub password: Option<String>,
    pub otp: Option<String>,
}

impl DetectedFields {
    pub fn is_empty(&self) -> bool {
        self.username.is_none() && self.password.is_none() && self.otp.is_none()
    }
}

#[derive(Debug, Error)]
pub enum DetectError {
    #[error("field detection failed: {0}")]
    Failed(String),
    #[error(transparent)]
    Driver(#[from] DriverError),
}
