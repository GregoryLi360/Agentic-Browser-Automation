//! In-memory caching decorator. Wraps any [`PasswordManager`] and loads its items once,
//! then answers every later read from process memory — `bw serve`-like speed with no
//! exposed port. `unlock` invalidates the cache.

use std::cell::RefCell;

use crate::source::password_manager::{ManagerError, PasswordManager, Status};
use crate::model::Item;

pub struct Cached<M: PasswordManager> {
    inner: M,
    items: RefCell<Option<Vec<Item>>>,
}

impl<M: PasswordManager> Cached<M> {
    pub fn new(inner: M) -> Self {
        Cached { inner, items: RefCell::new(None) }
    }
}

impl<M: PasswordManager> PasswordManager for Cached<M> {
    fn status(&self) -> Result<Status, ManagerError> {
        self.inner.status()
    }

    fn unlock(&self) -> Result<(), ManagerError> {
        let result = self.inner.unlock();
        *self.items.borrow_mut() = None;
        result
    }

    fn items(&self) -> Result<Vec<Item>, ManagerError> {
        if self.items.borrow().is_none() {
            let loaded = self.inner.items()?;
            *self.items.borrow_mut() = Some(loaded);
        }
        Ok(self.items.borrow().as_ref().expect("just populated").clone())
    }
}
