//! `vault-core` — the interface crate. Domain model + the provider *slots* a broker
//! composes, and the orchestrator that drives them. **No concrete backends live here;**
//! an impl crate (e.g. `vault`) supplies one implementation per slot — one cell of the
//! provider matrix.
//!
//! Layers depend only downward:
//!
//! ```text
//! model  ←  source{password_manager · otp · verification} · surface · flow · policy · passkey  ←  broker
//! ```
//!
//! The broker is a **challenge-iterator authentication runtime**: it asks the `surface`
//! for the `next_challenge`, dispatches it to the matching satisfier (a `source` value +
//! a `surface` fill, a passkey assertion, a flow), and repeats until the surface reports
//! no more challenges. Arbitrarily long sequences fall out for free.
//!
//! - `model`  — the shared vocabulary: `Item`, `Credential`, `Secret`, `Origin`, `Target`.
//! - `source` — where a fill-value comes from; three provider traits behind one parent,
//!   re-exported so callers say `source::PasswordManager` (no stutter):
//!     - `PasswordManager`   — hand back stored items, manage its session.
//!     - `OtpGenerator`      — generate a current code from a stored TOTP seed.
//!     - `VerificationSource`— read delivered email / SMS codes.
//!
//!   `Cached`, a backend-agnostic decorator, is the one concrete type that ships here.
//! - `surface`— `Surface`: the thing being signed into. Detects the `Challenge` it
//!   currently presents and applies values (`fill`/`submit`). Surface-agnostic — no JS,
//!   no CSS, no DOM in the contract (those are web-impl details). This is where the hard
//!   problem (challenge detection) lives.
//! - `flow`   — `Flow`: satisfies a `Challenge::Federated` (OAuth/SSO, magic links) — a
//!   delegated sign-in whose outcome is a session, not filled fields. Seam only.
//! - `policy` — `TargetPolicy`: authorize targets (allowlist + surface binding).
//! - `passkey`— `PasskeyAuthenticator`: satisfies a `Challenge::Passkey` (the WebAuthn
//!   assertion ceremony — a passkey is never typed).
//! - `broker` — `BrokerService`: the runtime. Composes a source, a surface, an otp
//!   generator, and a policy; registers any number of `Flow`s, plus optional verification
//!   and passkey satisfiers. Swap any slot's impl for a different matrix cell.

pub mod broker;
pub mod flow;
pub mod model;
pub mod passkey;
pub mod policy;
pub mod source;
pub mod surface;
