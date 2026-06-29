# browse v3 — Subscription-Driven Browser Automation

Drive a real browser (Firefox / Chromium / WebKit) in natural language, with the
reasoning loop running on your **coding-agent subscription** (Claude Code or Codex) —
no second model, no API key, no per-token bill. Log into sites **without ever exposing
credentials to the agent**: a small broker fills username / password / TOTP from
Bitwarden and the agent only ever sees `{filled: [...]}`.

Design rationale, security model, and the decision log live in [DESIGN.md](./DESIGN.md).

---

## Prerequisites

- **Platform:** a unix-like system — **macOS** or **Linux**. On **Windows**, do everything
  inside **WSL**; the launcher, the `~/.bw_session` path, and the `bw` / `playwright-cli`
  lookups assume a unix shell (native PowerShell/cmd is unsupported).
- **Node.js** 18+ (20+ recommended) — runs playwright-cli and the `bw` CLI.
- **Rust** toolchain (`cargo`) — builds the broker.
- A **coding-agent subscription:** Claude Code (Pro/Max) or Codex (ChatGPT Plus/Pro).
- A **Bitwarden** account, or a self-hosted **Vaultwarden** (fully open).

## Setup

### 1. System tools (per OS)

**macOS** (Homebrew):
```bash
brew install node rust bitwarden-cli
```

**Linux** (Debian/Ubuntu — adjust for your distro):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh        # rust
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -     # node 20 (or use nvm)
sudo apt-get install -y nodejs
npm install -g @bitwarden/cli                                         # bw
```

**Windows:** install WSL once (`wsl --install` in an admin PowerShell), open your distro,
then follow the **Linux** steps inside it.

### 2. Browser tooling + engine

```bash
npm install -g @playwright/cli@latest
playwright-cli install            # workspace
playwright-cli install --skills   # installs the agent skill (claude)
```

Install the browser engine you'll drive:

| Browser | Command | Notes |
|---|---|---|
| **Firefox** | `playwright-cli install-browser firefox` | the default we test against |
| Chromium | `playwright-cli install-browser chromium` | supported, untested here |
| WebKit | `playwright-cli install-browser webkit` | supported, untested here |

On **Linux / WSL**, append `--with-deps` to install the OS libraries the browser needs
(uses `sudo`), e.g. `playwright-cli install-browser firefox --with-deps`. For a *visible*
(`--headed`) window under WSL you need **WSLg** (Windows 11); on Windows 10 WSL run
headless (omit `--headed`).

### 3. Build the broker

```bash
(cd v3/vault-rs && cargo build --release)   # -> v3/vault-rs/target/release/vault
```

### 4. Bitwarden — log in + unlock

```bash
bw config server https://vault.bitwarden.com   # or your Vaultwarden URL
bw login                                        # email + master password [+ 2FA]
v3/vault unlock                                 # saves the session to ~/.bw_session (chmod 600)
```

### 5. Authenticate the agent on the subscription

Run `claude` (or Codex "Sign in with ChatGPT") and make sure `ANTHROPIC_API_KEY` /
`OPENAI_API_KEY` are **unset** — a present key silently shadows the subscription.

## Driving the browser

The agent calls `playwright-cli` through its shell tool; you just ask in English:

```
Open https://news.ycombinator.com in Firefox, snapshot it, and list the top 5 stories.
```

Manual equivalents:

```bash
playwright-cli -s=work open https://example.com --browser=firefox --headed
playwright-cli -s=work snapshot          # accessibility tree with refs (e5, e21, ...)
playwright-cli -s=work click e21
playwright-cli -s=work fill e3 "value" --submit
playwright-cli -s=work close
```

## Logging in without exposing secrets

The agent never sees your credentials — the broker reads them from Bitwarden and fills
them at the browser layer.

```bash
# 1. Unlock once (saves the session to ~/.bw_session). Run in a real terminal.
v3/vault unlock
#    Alternatively run `bw serve --port 8087` and the broker uses that instead.

# 2. Agent navigates to the login page (playwright-cli), then:
v3/vault grant www.example.com -s work --submit     # fills username + password
v3/vault otp   www.example.com -s work --submit      # fills a TOTP code, if the item has one
```

`grant` verifies the live page is actually on that origin before filling (refuses
otherwise), so credentials can't be coaxed into the wrong site.

### `vault` commands

| Command | Does | Returns |
|---|---|---|
| `vault grant <origin> [-s S] [--submit]` | fill username + password | `{filled:[...], totp_available}` |
| `vault otp <origin> [-s S] [--submit]` | fill a freshly generated TOTP | `{filled:["totp"]}` |
| `vault list` | login names + URIs | no secrets |
| `vault status` | backend (serve/cli) + unlock state | — |
| `vault unlock` | `bw unlock` → save session to `~/.bw_session` | — |

Output is one-line JSON; **secret values are never printed or returned**. Exit 0/1.
Optional `vault.allow` (one host per line) restricts which origins may be granted.

## Testing

```bash
(cd v3/vault-rs && cargo test)               # TOTP vs RFC 6238 vectors + notes parsing
# Live login fill on a public React form (Firefox):
playwright-cli -s=vaulttest open https://www.saucedemo.com --browser=firefox --headed
(cd v3/vault-rs && cargo run --example fill_live)
playwright-cli -s=vaulttest close
```

## Layout

```
v3/
  vault                 bash launcher -> the built Rust binary
  vault-rs/             Rust broker (cargo)
    src/
      main.rs           clap CLI: grant / otp / list / status / unlock
      bw.rs             Bitwarden backends: bw serve REST + bw CLI/~/.bw_session
      pw.rs             playwright-cli driver (fill, eval)
      fields.rs         login/OTP field detection
      otp.rs            TOTP via totp-rs (in-process; seed never hits an argv)
      origin.rs         origin binding + allowlist
      config.rs         env-overridable settings
    examples/fill_live.rs   live browser-fill check
  DESIGN.md             rationale, security model, decision log
```

## Security model (short)

Bitwarden owns storage, encryption, and the unlock session. The broker is stateless
glue: it reads a credential, checks the page origin, fills via Playwright, and returns
only field names. The reasoning agent is given the `vault` command but **not** `bw` or
the session — so the secret never enters its context. Full model + residual risks
(e.g. the secret transiting `playwright-cli` argv, passkey support) in [DESIGN.md](./DESIGN.md).

### Locking the agent to the broker (deployment)

The goal is that the agent can *use* credentials but never read a **plaintext** one —
any password that reaches the model's context is burned and must be rotated. The broker
already guarantees this on its own interface (no verb returns a value). To also stop the
agent from reaching a plaintext credential by another path, block it from running `bw`
directly **on the system where the browsing agent runs** (not needed on a dev box). The
broker is unaffected: `vault` spawns `bw` as a child process, invisible to the
permission layer.

Claude Code — add to `.claude/settings.json`:

```json
{
  "permissions": { "deny": ["Bash(bw:*)"] },
  "hooks": { "PreToolUse": [ { "matcher": "Bash", "hooks": [
    { "type": "command",
      "command": "jq -r '.tool_input.command' | grep -Eq '(^|[/[:space:]])bw([[:space:]]|$)|:8087' && { echo 'bw is broker-only — use vault' >&2; exit 2; }; exit 0" }
  ] } ] }
}
```

The hook also catches absolute-path `bw`, subshells, and `:8087` (bw serve), and fires
even under `--dangerously-skip-permissions` (where `deny` does not). Also keep `bw serve`
unexposed — it is an unauthenticated localhost API that returns plaintext. Codex has no
per-command deny; use its sandbox + approval policy instead.
