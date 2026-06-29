//! Web surface internals: a [`PageDriver`] (JS-eval transport) and a swappable
//! [`Detector`]. These are web-specific — `eval` and CSS selectors live *here*, not in
//! the `vault_core` contract. [`WebFormFiller`](crate::web_filler::WebFormFiller)
//! composes them into a `vault_core::fill::FormFiller`.

pub mod detector;
pub mod driver;

pub use detector::{DetectError, DetectedFields, Detector};
pub use driver::{DriverError, PageDriver};
