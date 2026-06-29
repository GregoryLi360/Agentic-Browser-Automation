//! Broker CLI — the only surface the reasoning agent touches.
//!
//! `auth` runs the challenge-iterator runtime: it detects each authentication challenge
//! the page presents and satisfies it (password, OTP, passkey, federated), looping until
//! the page is satisfied. Output is always one-line JSON; secret values are NEVER printed.
//! Exit 0 on success, 1 on any failure.

use clap::{Parser, Subcommand};
use serde_json::{json, Value};

use vault::default_broker;
use vault_core::broker::{AuthOptions, AuthOutcome, Broker, BrokerError};
use vault_core::model::{CredentialKind, ItemSummary, Target};
use vault_core::source::Status;
use vault_core::surface::Challenge;

#[derive(Parser)]
#[command(name = "vault", about = "Request-only credential broker.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Sign in to a target: detect each challenge the page presents and satisfy it,
    /// looping until done (or an out-of-band step is reached).
    Auth {
        origin: String,
        #[arg(short, long, env = "VAULT_PW_SESSION", default_value = "default")]
        session: String,
        /// Fill only; do not submit each step (the caller submits).
        #[arg(long)]
        no_submit: bool,
        /// Skip the page-target == requested-target binding check.
        #[arg(long = "no-verify-url")]
        no_verify_url: bool,
        /// Force the first step instead of detecting it: `password`, `otp`, `passkey`,
        /// or `sso:<provider>`. The loop then continues by detection.
        #[arg(long)]
        via: Option<String>,
    },
    /// List login names + URLs (no secrets).
    List,
    /// Password-manager reachability / unlock state.
    Status,
    /// Unlock the password manager (prompts for the master password).
    Unlock,
}

fn status_str(status: Status) -> &'static str {
    match status {
        Status::Unlocked => "unlocked",
        Status::Locked => "locked",
        Status::LoggedOut => "logged_out",
        Status::Unreachable => "unreachable",
    }
}

fn kind_str(kind: CredentialKind) -> &'static str {
    match kind {
        CredentialKind::BasicAuth => "basic-auth",
        CredentialKind::Totp => "totp",
        CredentialKind::Passkey => "passkey",
    }
}

fn challenge_str(challenge: &Challenge) -> String {
    match challenge {
        Challenge::Password => "password".into(),
        Challenge::OtpCode => "otp".into(),
        Challenge::Passkey => "passkey".into(),
        Challenge::Federated { provider } => format!("sso:{provider}"),
        Challenge::Approval { kind } => format!("approval:{kind}"),
    }
}

/// Parse a `--via` string into a forced first [`Challenge`].
fn parse_via(via: &str) -> Result<Challenge, String> {
    if let Some(provider) = via.strip_prefix("sso:") {
        return Ok(Challenge::Federated { provider: provider.to_lowercase() });
    }
    match via {
        "password" => Ok(Challenge::Password),
        "otp" => Ok(Challenge::OtpCode),
        "passkey" => Ok(Challenge::Passkey),
        other => Err(format!("unknown --via step '{other}'")),
    }
}

fn summary_json(summary: ItemSummary) -> Value {
    json!({
        "name": summary.name,
        "urls": summary.urls,
        "kinds": summary.kinds.into_iter().map(kind_str).collect::<Vec<_>>(),
    })
}

fn outcome_json(outcome: AuthOutcome) -> Value {
    match outcome {
        AuthOutcome::Authenticated { steps } => json!({
            "authenticated": true,
            "steps": steps.iter().map(challenge_str).collect::<Vec<_>>(),
        }),
        AuthOutcome::Pending { waiting_on, steps } => json!({
            "authenticated": false,
            "pending": challenge_str(&waiting_on),
            "steps": steps.iter().map(challenge_str).collect::<Vec<_>>(),
        }),
    }
}

fn run(cmd: Cmd) -> Result<Value, BrokerError> {
    match cmd {
        Cmd::Auth { origin, session, no_submit, no_verify_url, via } => {
            let force = match via.as_deref().map(parse_via).transpose() {
                Ok(force) => force,
                Err(e) => {
                    println!("{}", json!({ "ok": false, "error": e }));
                    std::process::exit(1);
                }
            };
            let opts = AuthOptions { submit: !no_submit, skip_page_check: no_verify_url, force };
            Ok(outcome_json(default_broker(&session).authenticate(&Target::parse(&origin), opts)?))
        }
        Cmd::List => {
            let logins: Vec<Value> =
                default_broker("default").list()?.into_iter().map(summary_json).collect();
            Ok(json!({ "logins": logins }))
        }
        Cmd::Status => Ok(json!({ "status": status_str(default_broker("default").status()?) })),
        Cmd::Unlock => {
            default_broker("default").unlock()?;
            Ok(json!({ "unlocked": true }))
        }
    }
}

fn main() {
    match run(Cli::parse().cmd) {
        Ok(Value::Object(mut map)) => {
            map.insert("ok".into(), Value::Bool(true));
            println!("{}", Value::Object(map));
        }
        Ok(value) => println!("{}", json!({ "ok": true, "result": value })),
        Err(e) => {
            println!("{}", json!({ "ok": false, "error": e.to_string() }));
            std::process::exit(1);
        }
    }
}
