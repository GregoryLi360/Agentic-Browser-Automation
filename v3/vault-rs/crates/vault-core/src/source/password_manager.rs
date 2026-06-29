//! Password-manager layer: a credential source. Its single responsibility is to hand
//! back stored [`Item`]s and manage its own session — nothing about matching, policy,
//! OTP generation, or the page.

pub mod cached;
pub mod error;

pub use cached::Cached;
pub use error::ManagerError;

use crate::model::{Item, ItemSummary};

/// A read-only store of login [`Item`]s with a session lifecycle. Bitwarden, 1Password,
/// an OS keychain, etc. each implement this; the broker depends only on the trait.
pub trait PasswordManager {
    /// Backend reachability / lock state.
    fn status(&self) -> Result<Status, ManagerError>;

    /// Establish or refresh an unlocked session.
    fn unlock(&self) -> Result<(), ManagerError>;

    /// Every stored item, secrets included.
    fn items(&self) -> Result<Vec<Item>, ManagerError>;

    /// Secret-free listing. Defaults to summarizing [`items`](Self::items); a backend
    /// that can list without decrypting secrets should override this.
    fn list(&self) -> Result<Vec<ItemSummary>, ManagerError> {
        Ok(self.items()?.iter().map(Item::summary).collect())
    }
}

/// Backend reachability / lock state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Unlocked,
    Locked,
    LoggedOut,
    Unreachable,
}
