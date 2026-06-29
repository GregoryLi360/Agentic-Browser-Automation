//! Policy layer: decide which targets may be filled. Two independent gates — allowlist
//! membership, and surface binding (the live surface must be the target we were asked to
//! fill). This, not encryption, is what stops a credential being typed into the wrong
//! site or app.

pub mod error;

pub use error::PolicyError;

use crate::model::Target;

pub trait TargetPolicy {
    /// The requested target is permitted to be filled at all.
    fn authorize(&self, requested: &Target) -> Result<(), PolicyError>;

    /// The live surface's target is the one we were asked to fill.
    fn verify(&self, observed: &Target, requested: &Target) -> Result<(), PolicyError>;
}
