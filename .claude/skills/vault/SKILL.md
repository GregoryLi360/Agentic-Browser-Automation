---
name: vault
description: Use whenever you hit a login / sign-in form during browser automation. Signs in via the local credential broker so secrets never enter your context — it detects each challenge (password, TOTP, passkey) and satisfies it. Trigger on any login page, "enter password", or 2FA / one-time-code prompt.
---

# vault — logging in without handling credentials

When a page needs a login, you do NOT type, ask for, guess, or read credentials. A local
broker (`v3/vault`) reads them from Bitwarden and drives the sign-in directly. You only get
back which steps were satisfied — never a value. That is by design.

## Rules

- NEVER type a password or username yourself, and NEVER ask the user for one.
- NEVER run `bw`, read `~/.bw_session`, or call `localhost:8087`. The broker does that.
- NEVER dump network request/response bodies on a login flow — the password is in the
  POST body. Don't read it.
- The broker returning only `{authenticated, steps}` is intentional. You are not meant to
  see the credential; do not try to obtain it another way.

## How to log in

1. Navigate to the site's login page yourself, e.g.
   `playwright-cli -s=<session> open https://example.com/login --browser=firefox`
2. Run the broker — it loops through whatever the page asks (password → OTP → …),
   verifying the live page is on that origin first:
   `v3/vault auth <origin-host> -s <session>`
   e.g. `v3/vault auth example.com -s work`
3. Continue with the now-authenticated session.

`auth` is a runtime: it detects each challenge and satisfies it, so a single call handles a
multi-step login (a password page, then a TOTP page, …). It submits each step by default —
pass `--no-submit` to fill without submitting.

For a step the detector can't see yet (a passkey prompt, a specific SSO button), force the
first step:
`v3/vault auth <origin> -s <session> --via passkey`   (or `--via sso:google`)

`<session>` must match the playwright-cli session you opened (`-s=`).

## Output

- `{"authenticated": true, "steps": ["password","otp"]}` — signed in.
- `{"authenticated": false, "pending": "approval:push", ...}` — stopped at an out-of-band
  step (push / QR); wait for the user to approve, then re-run.

## When it fails

- `no login found for '…'` → no saved credential for that origin. Tell the user; don't improvise.
- `surface '…' does not match requested target '…'` → you are not on that site. Navigate to
  the real login page.
- `vault is locked` → the user must run `v3/vault unlock` in a terminal. Don't attempt it.

## Commands

| Command | Does |
|---|---|
| `v3/vault auth <origin> -s <session> [--via <step>] [--no-submit]` | sign in: detect & satisfy each challenge (password, OTP, passkey, …) |
| `v3/vault list` | login names + URLs (no secrets) |
| `v3/vault status` | vault unlock state |
| `v3/vault unlock` | unlock the password manager (prompts for the master password) |
