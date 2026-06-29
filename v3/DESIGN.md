# v3 — Subscription-Driven Browser Automation

A Browser Use replacement for **personal/interactive** browsing that runs the reasoning loop on a **coding-agent subscription** (Claude Code or Codex) and uses **Microsoft's Playwright CLI** (`@playwright/cli`) as the browser tool layer. No second model, no separate LLM API key, no per-token bill — and Firefox-capable.

> Scope: for you, interactively, on your machine. Embedding in software, headless/CI, serving other users, or scaling to many parallel sessions requires API-key billing — see "When to switch to an API key."

---

## Verified install state (this machine, 2026-06-25)

Already installed and tested — you don't need to redo this unless reinstalling.

| Thing | State |
|---|---|
| `@playwright/cli` | v0.1.14 global (`~/.nvm/.../bin/playwright-cli`) |
| Skill | `<repo>/.claude/skills/playwright-cli/SKILL.md` (auto-discovered by Claude Code) |
| Workspace | initialized at repo root (`.playwright/`) |
| Firefox engine | `firefox-1530` (bound to cli's playwright-core 1.61.0-alpha) |
| Chrome | present, default |
| `ANTHROPIC_API_KEY` | unset (good — subscription is the loop, not an API key) |
| Smoke test | PASS — headed Firefox → HN → extracted top 5 stories → close |

Gitignored: `.playwright/`, `.playwright-cli/`, `*.state.json`, `auth.json`.

**Real command surface differs slightly from the original handoff prose:**
- Install browser: `playwright-cli install-browser firefox` (positional, not `--browser=`).
- Install skill: `playwright-cli install --skills` (target `claude` is default).
- Sessions are named with `-s=<name>`; `open` starts a persistent browser daemon, later commands attach.
- Element refs come from `snapshot` (e.g. `e15`); `--raw eval "<js>"` is cleanest for structured extraction.
- `--persistent` writes the profile to disk; `--profile=<dir>` pins a directory.

---

## The decision (why this stack)

- **`@playwright/cli` is brainless:** shell commands (`open`, `snapshot`, `click e5`, `fill e3 "x"`). The reasoning comes from whatever calls it. When the caller is Claude Code / Codex on your subscription, the subscription *is* the loop — one model, covered by the seat you already pay for.
- **Token-efficient by design.** Saves page snapshots to disk; the agent reads only what it needs instead of streaming the whole accessibility tree every step. ~27K tokens/task vs ~114K for Playwright MCP — ~4x reduction, larger on long sessions. On a subscription that's rate-limit budget saved.
- **Cross-browser, incl. Firefox.** Chromium, Firefox, WebKit via one install flag. (Why playwright-cli over agent-browser, which is Chromium/Safari-only.)
- **Same skill on both agents.** Microsoft's skill is agent-agnostic — identical setup on Claude Code or Codex.

Browser Use's only edge is packaging for productization (importable library, embedded/headless) — which needs an API key anyway, so irrelevant for personal use.

---

## Run it

Drive in natural language; the agent calls the CLI through its shell tool. Manual form:

```bash
playwright-cli -s=work open https://example.com --browser=firefox --headed --persistent
playwright-cli -s=work snapshot                 # accessibility tree with refs (e5, e21...)
playwright-cli -s=work click e21
playwright-cli -s=work fill e3 "value" --submit
playwright-cli -s=work --raw eval "document.title"
playwright-cli -s=work screenshot --filename=page.png
playwright-cli list                             # running sessions
playwright-cli -s=work close
```

---

## Logins & credentials (current: dummy accounts)

Create dedicated **dummy accounts** for the agent, never your real ones. Sessions are state — authenticate once, reuse.

- **Persist to disk** so no re-login per run:
  ```bash
  playwright-cli -s=work open https://app.example.com --browser=firefox --headed --persistent
  # log in manually once in the visible window; profile is saved
  ```
- **Or snapshot auth state to a file** (reproducible / portable):
  ```bash
  playwright-cli -s=work state-save logged-in.json
  playwright-cli -s=work state-load logged-in.json
  ```
- Separate profile per dummy account to isolate sessions.

Keep first-time login human-in-the-loop (MFA / CAPTCHA / passkeys require a human). **Do not paste real passwords into prompts** — prompt text rides along in tool calls and lands in model-provider payloads and logs. Dummy accounts sidestep this; treat their creds as low-value.

---

## Token / rate-limit notes

No per-token bill now, but browsing is token-heavy and draws on your subscription's usage window. Keep lean:
- Disk-snapshot model keeps per-step cost roughly flat even on 50–100 step sessions (vs MCP, where stale snapshots pile up and the agent hallucinates elements by ~step 15).
- Read only the snapshot sections needed; avoid re-snapshotting when waiting on one element suffices; minimize navigations; use `--raw eval` for targeted extraction.

---

## Phase 2 — Credential broker (later)

When dummy accounts work and you want real accounts / multi-site / audited access, add a **credential broker**. Contract:

- **Secret never enters the agent's context.** Agent calls `login(origin)`; broker checks allowlist, performs fill+submit **at the browser layer**, returns only `ok / failed`. Never hand the password back to the agent.
- **Per-origin allowlist** — a credential only unlocks its bound site.
- **Audit log + optional human-approval gate** per attempt; rotation/revocation independent of the agent.
- **Prefer pre-seeded `state-load` over live login** where a session can be captured once; the agent then never touches a credential.
- **Harden the broker**: localhost-only, callable but not readable by the agent; it becomes the highest-value target.
- **Don't broker the un-brokerable**: TOTP can be injected if you choose; passkeys/WebAuthn are hardware/biometric-bound — leave human.

A broker controls *which* logins happen, not what the session then does. Pair with **post-login controls** against prompt injection (logged-in agent + hostile page = acts as you):
- navigation allowlist, human approval on writes/irreversible actions, content/output boundaries so page text isn't read as instructions.

---

## When to switch to an API key

Subscription covers **you, interactively, on your machine, for yourself.** Cross any of these → API-key billing (required per provider terms):
- Calling the loop programmatically / unattended from your own software.
- Headless on a server, in CI, or a scheduled job with no interactive login.
- Output consumed by anyone other than you (product, team bot, client deliverable).
- Many parallel/independent sessions at scale.

Then use Playwright (or Browser Use) with an API key — per-token cost justified because you're shipping a product.

---

## TL;DR

1. Auth Claude Code / Codex on the subscription; `unset` the API-key env var. (done)
2. `npm i -g @playwright/cli@latest` → `install` → `install-browser firefox` → `install --skills`. (done)
3. Dummy accounts; persist with `--persistent` or `state-save`/`state-load`.
4. Drive in natural language; subscription is the loop, on Firefox.
5. Phase 2: credential broker (secret never enters agent context) + post-login action/navigation policy.
6. API key only when crossing into embedded / headless / multi-user / scaled use.
