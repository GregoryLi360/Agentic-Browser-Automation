//! Page driver: low-level control of the live page. Reads the page and sets field
//! values. Knows nothing about *which* fields are login fields — that is the detector's
//! job. The concrete driver wraps the browser transport (e.g. playwright-cli).

use thiserror::Error;

use vault_core::model::Origin;

pub trait PageDriver {
    /// The page's current top-level origin, for origin-binding checks.
    fn origin(&self) -> Result<Origin, DriverError>;

    /// Set the value of the element at `selector`, optionally submitting its form.
    fn fill(&self, selector: &str, value: &str, submit: bool) -> Result<(), DriverError>;

    /// Evaluate JS in the page and return its JSON result. The detector uses this to
    /// inspect the DOM.
    fn eval(&self, script: &str) -> Result<serde_json::Value, DriverError>;
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("browser page unavailable: {0}")]
    Unavailable(String),
    #[error("driver command failed: {0}")]
    Command(String),
}
