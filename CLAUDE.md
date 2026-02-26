# Agentic-Browser-Automation

Selenium browser agent that reuses your real browser profile (cookies, sessions, localStorage) and accepts commands via a file interface. Designed to be driven by LLM agents.

## Quick Reference

```bash
# Install
pip install selenium requests

# Start browser session (Firefox, auto-detects profile)
python3 browse.py https://example.com

# Start with Chrome
python3 browse.py https://example.com -b chrome

# Send a command (picked up within 1 second)
echo "navigate https://example.com/dashboard" > commands.txt

# Read the result
cat result.txt

# Read saved HTML snapshot
cat data/<timestamp>_nav.html
```

## Project Structure

```
browse.py       — Main browser agent. Launches Selenium, polls commands.txt, saves HTML snapshots.
cookies.py      — Cookie extraction library (from yt-dlp). Supports Firefox, Chrome, Safari, Edge, Brave, etc.
fetch.py        — Lightweight HTTP fetch using extracted browser cookies (no Selenium needed).
commands.txt    — Write commands here (auto-deleted after processing). Gitignored.
result.txt      — JSON result of last command. Gitignored.
data/           — Timestamped HTML snapshots. Gitignored.
```

## Agent Workflow (browse.py)

This is the core loop for driving the browser as an AI agent:

1. **Start**: `python3 browse.py <url>` — launches browser, saves initial snapshot to `data/`
2. **Read snapshot**: Parse the HTML file referenced in `result.txt` → `snapshot` field
3. **Decide & act**: Write a command to `commands.txt`
4. **Wait**: Poll `result.txt` for a new timestamp (commands execute within ~1-3s)
5. **Loop**: Read new snapshot, decide next action, repeat

### Commands

| Command | Syntax | Notes |
|---|---|---|
| `navigate` | `navigate <url>` | Waits 15s for body + 2s settle |
| `click` | `click <css_selector>` | Waits 10s for clickable + 2s settle |
| `type` | `type <css_selector> <text>` | Clears field first, 1s settle |
| `select` | `select <css_selector> <value>` | Dropdown by value, 1s settle |
| `snapshot` | `snapshot` | Force-save current page HTML |
| `js` | `js <javascript_code>` | Execute JS, result in message field |

### Result Format (result.txt)

```json
{"status": "ok", "message": "Navigated to ...", "time": "2026-02-26T11:40:57", "snapshot": "data/20260226_114057_nav.html"}
```

Status is `"ok"` or `"error"`. On error, `message` contains the exception text.

## Key Implementation Details

- **commands.txt** is deleted after reading. Write all commands at once if batching.
- **Comments** (lines starting with `#`) are ignored.
- **CSS selectors** are used for all element targeting (not XPath).
- **Profile copy**: Browser profile is copied to a temp dir to avoid locking. Lock files are excluded.
- **Snapshots** are named `YYYYMMDD_HHMMSS_<label>.html` where label is: `init`, `nav`, `click`, `type`, `select`, `snap`, `js`.

## fetch.py — Cookied HTTP Requests

For simple HTTP fetches that don't need a full browser:

```python
from fetch import fetch
resp = fetch("https://example.com/api/data", browser="firefox")
print(resp.json())
```

```bash
python3 fetch.py https://example.com/api/data -b firefox -o output.json
```

## CLI Flags (browse.py)

| Flag | Default | Description |
|---|---|---|
| `url` (positional) | required | Starting URL |
| `-b, --browser` | `firefox` | `firefox` or `chrome` |
| `--profile` | auto-detect | Explicit profile path |
| `--no-profile` | false | Fresh session (no cookies) |
| `--headless` | false | No visible browser window |

## Environment Variables

| Variable | Purpose |
|---|---|
| `FIREFOX_PROFILE` | Override Firefox profile path |
| `CHROME_PROFILE` | Override Chrome profile path |

## Dependencies

- `selenium` (required)
- `requests` (for fetch.py)
- geckodriver (Firefox) or chromedriver (Chrome) on PATH

## Conventions

- Python 3, standard library style
- No type annotations in existing code
- Minimal dependencies — prefer stdlib
- File-based IPC (commands.txt / result.txt) — no sockets or APIs
- HTML snapshots are the primary way agents observe page state
