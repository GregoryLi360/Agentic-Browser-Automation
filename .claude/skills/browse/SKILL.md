---
name: browse
description: Drive a real browser (with your cookies/sessions) to automate web tasks. Uses Selenium with your actual browser profile.
when_to_use: "browse a website, scrape a page, automate a login, interact with a web app, fill a form, click through a site, web automation"
disable-model-invocation: true
argument-hint: "[url]"
allowed-tools: Bash(python3 browse.py *) Bash(python3 fetch.py *) Bash(./browse.py *) Bash(./fetch.py *) Bash(echo * > commands.txt) Bash(cat result.txt) Bash(cat data/*) Bash(until *) Bash(sleep *) Bash(tail *) Bash(grep *) Bash(kill *) Bash(ps *) Read(*)
---

# Browser Automation Skill

Drive a real browser with your actual cookies/sessions via `browse.py`.

## Setup

Requires: `selenium`, `requests`, geckodriver (Firefox) or chromedriver (Chrome) on PATH.

## Launch (IMPORTANT: be patient, profile copy takes 30-90s)

```bash
python3 browse.py <URL>
# Run with run_in_background: true
```

Wait for ready marker before sending commands:
```bash
until grep -qE "Watching|Traceback|Error|Exception" /path/to/task.output 2>/dev/null; do
  sleep 5
done
tail -20 /path/to/task.output
```

Ready: `Watching commands.txt for commands...`
Failure: `Traceback`, `Error`, `Exception`

## Sending Commands

Write to `commands.txt` in the repo root:

```bash
echo "navigate https://example.com" > commands.txt
sleep 3
cat result.txt
```

## Available Commands

| Command | Syntax | Notes |
|---|---|---|
| navigate | `navigate <url>` | Waits 15s for body + 2s settle |
| click | `click <css_selector>` | Waits 10s for clickable + 2s settle |
| type | `type <css_selector> <text>` | Clears field first, 1s settle |
| select | `select <css_selector> <value>` | Dropdown by value |
| snapshot | `snapshot` | Force-save current page HTML |
| js | `js <javascript_code>` | Execute JS, result in message field |

## Result Format (result.txt)

```json
{"status": "ok", "message": "...", "time": "...", "snapshot": "data/20260226_114057_nav.html"}
```

## Agent Loop

1. Launch browse.py in background, wait for ready
2. Read initial snapshot from result.txt → snapshot path
3. Read the HTML snapshot to understand page state
4. Decide action, write command to commands.txt
5. Wait ~3s, read result.txt
6. Read new snapshot, repeat

## CSS Selector Patterns

```
#login-button              — by ID
.submit-btn                — by class
button[type="submit"]      — by attribute
input[name="email"]        — by name
[data-testid="submit"]     — by data attribute
.modal .confirm-btn        — nested
ul.menu li:nth-child(3) a  — positional
```

## JavaScript for Complex Cases

```
js return document.title
js window.scrollTo(0, document.body.scrollHeight)
js document.querySelector('.target').scrollIntoView()
js return JSON.stringify(Array.from(document.querySelectorAll('.item')).map(el => el.textContent))
```

## CLI Flags

| Flag | Default | Description |
|---|---|---|
| url (positional) | required | Starting URL |
| -b, --browser | auto-detect | `firefox` or `chrome` |
| --profile | auto-detect | Explicit profile path |
| --no-profile | false | Fresh session (no cookies) |
| --headless | false | No visible window |

## fetch.py — Simple HTTP with Browser Cookies

When you only need to call an API (no clicking/typing), use fetch.py instead:

```bash
python3 fetch.py https://api.example.com/data -b firefox -o output.json
```

## Critical Rules

- **NEVER kill firefox or chrome processes** — only kill browse.py, geckodriver, chromedriver
- **commands.txt is deleted after reading** — write all commands at once if batching
- **Always read snapshots** to understand page state, don't guess
- **CSS selectors only** (not XPath)
- Profile copy to temp dir can take 30-90s — do NOT abort early

## Cleanup

```bash
# Find browse.py processes
ps aux | grep browse.py

# Kill only browse.py and drivers, NEVER the browser itself
kill <browse.py PID>
pkill geckodriver   # or chromedriver
```

## Troubleshooting

- **Element not interactable**: Scroll into view with `js document.querySelector('.target').scrollIntoView()`, then retry
- **Element not found**: Take a snapshot, read HTML to find correct selector
- **Timeout**: Check if site needs different auth or URL is wrong
- **Dynamic content not loaded**: Use `js return document.querySelector('.content')?.textContent?.length > 0` to poll
