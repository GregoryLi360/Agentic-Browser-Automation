# Agentic-Browser-Automation

A Selenium browser agent that reuses your real browser profile (cookies, localStorage, sessions) and accepts commands via a file. Supports Firefox and Chrome. Designed to be driven by LLM agents like Claude Code.

## Table of contents

- [Why](#why)
- [Quickstart](#quickstart)
- [How it works](#how-it-works)
- [Browser options](#browser-options)
- [Commands](#commands)
- [Output](#output)
- [Other tools](#other-tools)

## Why

Hooking an agentic LLM up to a browser UI — especially one where you're already logged in — is harder than it should be. Perplexity's Comet ballooned into a RAM-hog; OpenClaw burned tokens for little progress. Heavy "AI browser" apps own the whole stack and pay for it.

This repo flips it: let Selenium drive a real browser using your existing profile (cookies, sessions, localStorage), dump each page to a static `.html` snapshot, and let any coding agent (Claude Code, etc.) read snapshots and write commands via a file. The agent doesn't need browser bindings — just file I/O. The browser doesn't need an LLM embedded — just a polling loop.

Example use cases:

- **Scraping behind anti-bot defenses** — e.g. Amazon listings, where a real logged-in profile + page-by-page navigation slips past detection that headless scrapers trip.
- **Working inside auth-walled admin portals** — review applications, click through dashboards, fill forms in tools that require an active session cookie.
- **Any LLM-driven workflow on a site you're already logged into** — the agent reads HTML snapshots and writes commands; no extra auth wiring.

## Quickstart

```bash
pip install selenium
# Firefox: geckodriver on PATH — https://github.com/mozilla/geckodriver/releases
# Chrome: chromedriver on PATH — https://googlechromelabs.github.io/chrome-for-testing/

chmod +x browse.py fetch.py

# Open a page with your Firefox cookies (profile auto-detected)
./browse.py https://example.com

# Or use Chrome
./browse.py https://example.com -b chrome

# Send a command
echo "navigate https://example.com/dashboard" > commands.txt

# Check the result
cat result.txt

# Read the saved HTML
ls data/
```

## How it works

1. `browse.py` copies your browser profile to a temp dir (so the browser can stay open) and launches a Selenium session
2. It watches `commands.txt` for instructions, executes them, and saves HTML snapshots to `data/`
3. Results are written to `result.txt` as JSON
4. An external agent reads the HTML, decides what to do next, and writes more commands

## Browser options

```bash
# Firefox (default)
python3 browse.py https://example.com
python3 browse.py https://example.com -b firefox

# Chrome
python3 browse.py https://example.com -b chrome

# Specify a profile path
python3 browse.py https://example.com --profile "/path/to/profile"

# Or use environment variables
export FIREFOX_PROFILE="/path/to/firefox/profile"
export CHROME_PROFILE="/path/to/chrome/user-data-dir"

# Fresh profile (no cookies/auth)
python3 browse.py https://example.com --no-profile

# Headless mode (no visible browser window)
python3 browse.py https://example.com --headless
```

Profile resolution order: `--profile` flag > env var (`FIREFOX_PROFILE` / `CHROME_PROFILE`) > auto-detect.

| Browser | Driver | Auto-detect locations |
|---|---|---|
| Firefox | [geckodriver](https://github.com/mozilla/geckodriver/releases) | `~/Library/Application Support/Firefox/Profiles/*.default-release` (macOS), `~/.mozilla/firefox/*.default-release` (Linux) |
| Chrome | [chromedriver](https://googlechromelabs.github.io/chrome-for-testing/) | `~/Library/Application Support/Google/Chrome` (macOS), `~/.config/google-chrome` (Linux) |

## Commands

Write to `commands.txt` (one per line, picked up within 1 second):

| Command | Example | Description |
|---|---|---|
| `navigate <url>` | `navigate https://example.com` | Go to a URL |
| `click <selector>` | `click button.submit` | Click an element (CSS selector) |
| `type <selector> <text>` | `type #email user@example.com` | Type into an input |
| `select <selector> <value>` | `select #country US` | Select a dropdown value |
| `snapshot` | `snapshot` | Save current page HTML |
| `js <code>` | `js return document.title` | Execute JavaScript |

## Output

- **`data/`** — timestamped HTML snapshots after each command
- **`result.txt`** — JSON result of the last command

```json
{"status": "ok", "message": "Navigated to https://example.com", "time": "...", "snapshot": "data/20260224_001234_nav.html"}
```

## Other tools

- **`fetch.py`** — fetch a URL using extracted browser cookies (no Selenium needed)
- **`cookies.py`** — cookie extraction library (from yt-dlp)
