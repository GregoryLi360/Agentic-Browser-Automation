//! Default authentication surface — the impl of `vault_core::surface::Surface` for the web.
//!
//! [`WebSurface`] detects the current [`Challenge`](vault_core::surface::Challenge) with a
//! [`HeuristicDetector`] and applies values via a [`PlaywrightDriver`]. The `page`
//! submodule holds those web-internal traits — `eval`/CSS live there, never in the
//! `vault_core` contract.

pub mod heuristic;
pub mod page;
pub mod playwright;
pub mod web;

pub use heuristic::HeuristicDetector;
pub use playwright::PlaywrightDriver;
pub use web::WebSurface;
