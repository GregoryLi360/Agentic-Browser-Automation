# Project Context — Agentic-Browser-Automation

> This file provides full context for AI agents working in this repo.
> For a quick reference, see [CLAUDE.md](./CLAUDE.md).
> For detailed skill docs, see the [skills/](./skills/) folder.

## What This Project Is

A file-based browser automation system designed to be driven by LLM agents. It lets an AI agent control a real browser session (with your actual cookies and authentication) through a simple text interface:

- **Write** commands to `commands.txt`
- **Read** results from `result.txt` and HTML snapshots from `data/`

No API server, no WebSocket — just files. This makes it trivially easy for any agent framework to integrate.

## Architecture

```
┌─────────────┐     commands.txt     ┌──────────────┐     Selenium      ┌─────────┐
│   AI Agent   │ ──────────────────> │   browse.py   │ ───────────────> │ Browser  │
│ (Claude, etc)│ <────────────────── │  (main loop)  │ <─────────────── │ (Firefox │
└─────────────┘   result.txt + HTML  └──────────────┘    page source    │  Chrome) │
                                                                         └─────────┘
```

### Data Flow

1. Agent writes `navigate https://site.com` to `commands.txt`
2. `browse.py` detects the file within 1 second, deletes it, executes the command
3. Selenium drives the browser, waits for page load
4. Page HTML is saved to `data/YYYYMMDD_HHMMSS_nav.html`
5. Result JSON is written to `result.txt` with status + snapshot path
6. Agent reads `result.txt`, then reads the snapshot HTML to understand page state
7. Agent decides next action, writes next command — loop continues

## File Roles

| File | Role | Created by | Read by |
|---|---|---|---|
| `browse.py` | Main browser agent | developer | system |
| `cookies.py` | Cookie extraction library | developer | browse.py, fetch.py |
| `fetch.py` | HTTP fetch with browser cookies | developer | agent |
| `commands.txt` | Command input | agent | browse.py (then deleted) |
| `result.txt` | Command result output | browse.py | agent |
| `data/*.html` | Page snapshots | browse.py | agent |

## Available Commands

```
navigate <url>                  → go to URL, save snapshot
click <css_selector>            → click element, save snapshot
type <css_selector> <text>      → clear + type into input, save snapshot
select <css_selector> <value>   → select dropdown option, save snapshot
snapshot                        → save current page HTML
js <javascript_code>            → execute JS, return result, save snapshot
```

All element selectors are **CSS selectors** (not XPath).

## Result Format

```json
{
  "status": "ok",
  "message": "Navigated to https://example.com",
  "time": "2026-02-26T11:40:57.111645",
  "snapshot": "data/20260226_114057_nav.html"
}
```

On error: `"status": "error"` and `message` contains the error detail.

## Typical Agent Session

```bash
# 1. Start the browser (run in background or separate terminal)
python3 browse.py https://example.com -b firefox &

# 2. Read initial snapshot
cat result.txt
# → {"status": "ok", "snapshot": "data/20260226_114057_init.html", ...}

# 3. Read the HTML to understand the page
cat data/20260226_114057_init.html

# 4. Interact
echo "click a.login-link" > commands.txt
sleep 3
cat result.txt
cat data/20260226_114100_click.html

echo "type #email user@example.com" > commands.txt
sleep 2
echo "type #password secret123" > commands.txt
sleep 2
echo "click button[type=submit]" > commands.txt
sleep 3
cat result.txt
```

## Skills Reference

Detailed guides in the `skills/` folder:

- **[browser-automation.md](./skills/browser-automation.md)** — Full guide to driving browse.py as an agent, CSS selector patterns, JS execution, error handling
- **[cookie-extraction.md](./skills/cookie-extraction.md)** — Using cookies.py and fetch.py for authenticated HTTP requests without Selenium
- **[troubleshooting.md](./skills/troubleshooting.md)** — Common issues: driver setup, profile detection, element targeting, dynamic content

## Development Notes

- Python 3, no type annotations, minimal dependencies
- Only two runtime deps: `selenium`, `requests`
- `cookies.py` is adapted from yt-dlp — treat it as a vendored dependency, avoid modifying
- File-based IPC is intentional — keeps the interface simple for any agent framework
- HTML snapshots are the agent's "eyes" — they contain the full page source after each action
