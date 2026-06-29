//! `vault-core` — the interface crate for an agent-facing credential broker: a domain
//! model, the provider trait *slots* a broker composes, and the authentication runtime.
//! No concrete backends live here — an impl crate (e.g. `vault`) supplies one per slot.
//!
//! This first layer is `model`, the shared vocabulary every other layer speaks.

pub mod model;
