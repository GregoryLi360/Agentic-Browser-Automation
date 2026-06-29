//! Sensitive values, kept out of logs and zeroized on drop.

use std::fmt;

use zeroize::Zeroizing;

/// A sensitive value (password, TOTP seed, OTP code). Never `Display`ed, redacted in
/// `Debug`, and zeroized when dropped — the inner [`Zeroizing`] handles the wipe, so
/// there is no hand-rolled `Drop`. Borrow the raw bytes briefly via [`expose`].
///
/// [`expose`]: Secret::expose
#[derive(Clone)]
pub struct Secret(Zeroizing<String>);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Secret(Zeroizing::new(value.into()))
    }

    /// Borrow the raw value to use it (e.g. type it into a field). Keep the borrow short.
    pub fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Secret(Zeroizing::new(value))
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}
