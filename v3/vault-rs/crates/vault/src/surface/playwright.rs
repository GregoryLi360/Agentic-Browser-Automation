//! playwright-cli page driver. Fills go through Playwright's `locator.fill()`, which
//! sets the value via the native setter and dispatches input/change events so SPA login
//! forms register them.

use std::env;
use std::process::Command;

use serde_json::Value;

use vault_core::model::Origin;

use super::page::{DriverError, PageDriver};

pub struct PlaywrightDriver {
    bin: String,
    session: String,
}

impl PlaywrightDriver {
    /// `session` is the playwright-cli session name (`-s=<session>`). The binary is
    /// `VAULT_PW_BIN` or `playwright-cli`.
    pub fn new(session: impl Into<String>) -> Self {
        PlaywrightDriver {
            bin: env::var("VAULT_PW_BIN").unwrap_or_else(|_| "playwright-cli".into()),
            session: session.into(),
        }
    }

    fn run(&self, args: &[&str]) -> Result<String, DriverError> {
        let out = Command::new(&self.bin)
            .arg(format!("-s={}", self.session))
            .args(args)
            .output()
            .map_err(|e| DriverError::Unavailable(format!("could not run {}: {e}", self.bin)))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            let so = String::from_utf8_lossy(&out.stdout);
            let detail = if err.trim().is_empty() { so.trim() } else { err.trim() };
            return Err(DriverError::Command(format!("{} {}", args.first().unwrap_or(&""), detail)));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

impl PageDriver for PlaywrightDriver {
    fn origin(&self) -> Result<Origin, DriverError> {
        Ok(Origin::parse(self.eval("location.href")?.as_str().unwrap_or_default()))
    }

    fn fill(&self, selector: &str, value: &str, submit: bool) -> Result<(), DriverError> {
        let mut args = vec!["fill", selector, value];
        if submit {
            args.push("--submit");
        }
        self.run(&args).map(|_| ())
    }

    fn eval(&self, script: &str) -> Result<Value, DriverError> {
        let out = self.run(&["--raw", "eval", script])?;
        let trimmed = out.trim();
        let value: Value = serde_json::from_str(trimmed)
            .map_err(|e| DriverError::Command(format!("eval returned non-JSON: {e} ({trimmed})")))?;
        // `--raw` may double-encode: a JSON string whose contents are themselves JSON.
        if let Value::String(inner) = &value {
            if let Ok(decoded) = serde_json::from_str::<Value>(inner) {
                return Ok(decoded);
            }
        }
        Ok(value)
    }
}
