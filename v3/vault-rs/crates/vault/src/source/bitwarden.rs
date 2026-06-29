//! Bitwarden via the `bw` CLI. Talks to the local encrypted cache through
//! `bw --session` and maps Bitwarden's item JSON into the neutral
//! [`Item`]/[`Credential`] model. The session token is held in memory only — never
//! written to disk, and no `bw serve` port is ever opened.

use std::cell::RefCell;
use std::env;
use std::process::{Command, Stdio};

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

use vault_core::source::password_manager::{ManagerError, PasswordManager, Status};
use vault_core::model::{Credential, Item, Secret, Totp};

/// Bitwarden item `type` for a login (vs card/identity/note).
const TYPE_LOGIN: i64 = 1;

/// `bw` output substrings that mean "the session is no good, re-unlock".
const LOCKED: [&str; 4] = ["Vault is locked", "You are not logged in", "Invalid session", "mac failed"];

pub struct BitwardenCli {
    session: RefCell<Option<String>>,
}

impl Default for BitwardenCli {
    fn default() -> Self {
        Self::new()
    }
}

impl BitwardenCli {
    /// Adopts a `BW_SESSION` token from the environment if present. The token stays in
    /// memory; an absent or rejected one triggers `bw unlock` on first use.
    pub fn new() -> Self {
        let session = env::var("BW_SESSION").ok().filter(|s| !s.is_empty());
        BitwardenCli { session: RefCell::new(session) }
    }

    fn run(&self, args: &[&str]) -> Result<Vec<u8>, ManagerError> {
        let mut cmd = Command::new("bw");
        cmd.arg("--nointeraction");
        if let Some(token) = self.session.borrow().as_deref() {
            cmd.args(["--session", token]);
        }
        let out = cmd
            .args(args)
            .output()
            .map_err(|e| ManagerError::Unreachable(format!("could not run bw: {e}")))?;
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        if LOCKED.iter().any(|m| combined.contains(m)) {
            return Err(ManagerError::Locked);
        }
        if !out.status.success() {
            return Err(ManagerError::Backend(String::from_utf8_lossy(&out.stderr).trim().to_string()));
        }
        Ok(out.stdout)
    }

    /// Runs `bw`; if the session is missing/invalid, unlocks once and retries.
    fn call(&self, args: &[&str]) -> Result<Vec<u8>, ManagerError> {
        match self.run(args) {
            Err(ManagerError::Locked) => {
                self.unlock()?;
                self.run(args)
            }
            other => other,
        }
    }

    fn json<T: DeserializeOwned>(&self, args: &[&str]) -> Result<T, ManagerError> {
        let bytes = self.call(args)?;
        serde_json::from_slice(&bytes).map_err(|e| ManagerError::Backend(format!("bad bw json: {e}")))
    }
}

impl PasswordManager for BitwardenCli {
    fn status(&self) -> Result<Status, ManagerError> {
        // `bw status` answers without a session; report the state rather than re-unlock.
        let out = self.run(&["status"]).or_else(|e| match e {
            ManagerError::Locked => Ok(Vec::new()),
            other => Err(other),
        })?;
        let value: Value = serde_json::from_slice(&out).unwrap_or(Value::Null);
        Ok(match value.get("status").and_then(Value::as_str) {
            Some("unlocked") => Status::Unlocked,
            Some("locked") => Status::Locked,
            Some("unauthenticated") => Status::LoggedOut,
            _ => Status::Unreachable,
        })
    }

    fn unlock(&self) -> Result<(), ManagerError> {
        // stderr stays on the tty so bw can prompt; stdout (the raw token) is captured
        // and kept in memory only.
        let out = Command::new("bw")
            .args(["unlock", "--raw"])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| ManagerError::Unreachable(format!("could not run bw: {e}")))?;
        let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !out.status.success() || token.is_empty() {
            return Err(ManagerError::Unlock(
                "bw could not unlock — run `bw unlock` in a terminal, or check you are logged in".into(),
            ));
        }
        *self.session.borrow_mut() = Some(token);
        Ok(())
    }

    fn items(&self) -> Result<Vec<Item>, ManagerError> {
        let raw: Vec<RawItem> = self.json(&["list", "items"])?;
        Ok(raw.into_iter().filter_map(RawItem::into_item).collect())
    }
}

// ---- Bitwarden item JSON: only the fields we read ----

#[derive(Deserialize)]
struct RawItem {
    name: Option<String>,
    #[serde(rename = "type")]
    typ: Option<i64>,
    login: Option<RawLogin>,
}

#[derive(Deserialize)]
struct RawLogin {
    username: Option<String>,
    password: Option<String>,
    totp: Option<String>,
    uris: Option<Vec<RawUri>>,
}

#[derive(Deserialize)]
struct RawUri {
    uri: Option<String>,
}

impl RawItem {
    /// Map a Bitwarden login item into a neutral [`Item`], or `None` if it is not a
    /// login or carries no usable credential.
    fn into_item(self) -> Option<Item> {
        if self.typ != Some(TYPE_LOGIN) {
            return None;
        }
        let login = self.login?;
        let mut credentials = Vec::new();

        let has_basic = login.username.as_deref().is_some_and(|u| !u.is_empty())
            || login.password.as_deref().is_some_and(|p| !p.is_empty());
        if has_basic {
            credentials.push(Credential::BasicAuth {
                username: login.username.filter(|u| !u.is_empty()),
                password: Secret::new(login.password.unwrap_or_default()),
            });
        }
        if let Some(seed) = login.totp.filter(|t| !t.is_empty()) {
            credentials.push(Credential::Totp(Totp::from_field(&seed)));
        }
        // (Passkeys via fido2Credentials are modeled but not yet mapped.)
        if credentials.is_empty() {
            return None;
        }

        let urls = login.uris.unwrap_or_default().into_iter().filter_map(|u| u.uri).collect();
        Some(Item { name: self.name.unwrap_or_default(), urls, credentials })
    }
}
