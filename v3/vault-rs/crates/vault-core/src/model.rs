//! Shared kernel: the domain vocabulary every layer speaks. Types and their pure
//! methods only — no I/O, no policy, no orchestration.

pub mod item;
pub mod origin;
pub mod secret;
pub mod target;

pub use item::{Algorithm, Credential, CredentialKind, Item, ItemSummary, Passkey, Totp};
pub use origin::Origin;
pub use secret::Secret;
pub use target::Target;
