# Skill: Browser Automation Agent

How to drive `browse.py` as an AI agent to automate browser tasks with real authentication.

## Starting a Session

```bash
# Start in background so the agent can interact with it
python3 browse.py https://target-site.com &

# Or headless for non-visual tasks
python3 browse.py https://target-site.com --headless &
```

After startup, the initial page snapshot is saved. Check `result.txt` for the snapshot path.

## Agent Control Loop

```python
import json, time, os

def send_command(cmd):
    """Send a command and wait for the result."""
    with open("commands.txt", "w") as f:
        f.write(cmd + "\n")
    # Poll for result (command executes within 1s, plus page load time)
    time.sleep(3)
    with open("result.txt") as f:
        return json.load(f)

def read_snapshot(result):
    """Read the HTML snapshot from a command result."""
    if result.get("snapshot"):
        with open(result["snapshot"]) as f:
            return f.read()
    return None

# Example: navigate and extract data
result = send_command("navigate https://example.com/dashboard")
html = read_snapshot(result)
# ... parse html, decide next action ...

result = send_command("click button.load-more")
html = read_snapshot(result)
```

## Shell-Based Control (for Claude Code / CLI agents)

```bash
# Send command
echo "navigate https://example.com/login" > commands.txt

# Wait and check result
sleep 3
cat result.txt

# Read the snapshot
cat data/$(python3 -c "import json; print(json.load(open('result.txt'))['snapshot'])")

# Multi-step: login flow
echo "type #email user@example.com" > commands.txt
sleep 2
echo "type #password mypassword" > commands.txt
sleep 2
echo "click button[type=submit]" > commands.txt
sleep 3
cat result.txt
```

## CSS Selector Tips

All element targeting uses CSS selectors. Common patterns:

```
# By ID
click #login-button

# By class
click .submit-btn

# By tag + attribute
click button[type="submit"]
click input[name="email"]
click a[href="/dashboard"]

# By tag + class
click button.primary

# Nested selectors
click .modal .confirm-btn
click form#login input[type="submit"]

# nth-child
click ul.menu li:nth-child(3) a

# Data attributes
click [data-testid="submit"]
```

## JavaScript Execution

The `js` command runs arbitrary JavaScript and returns the result:

```
# Get page title
js return document.title

# Get text content
js return document.querySelector('.balance').textContent

# Scroll down
js window.scrollTo(0, document.body.scrollHeight)

# Wait for dynamic content
js return document.querySelectorAll('.item').length

# Extract structured data
js return JSON.stringify(Array.from(document.querySelectorAll('.product')).map(el => ({name: el.querySelector('.name').textContent, price: el.querySelector('.price').textContent})))

# Click something hard to target with CSS
js document.querySelector('shadow-host').shadowRoot.querySelector('button').click()
```

## Error Handling

When a command fails, `result.txt` will have `"status": "error"`:

```json
{"status": "error", "message": "click failed: Message: element not interactable", "time": "..."}
```

Common errors and fixes:
- **element not interactable**: Element is hidden or overlapped. Try scrolling (`js window.scrollBy(0, 300)`) or waiting.
- **no such element**: Selector doesn't match. Take a snapshot and inspect the HTML.
- **timeout**: Page didn't load in time. The page may require authentication or the URL may be wrong.

## Best Practices

1. **Always read snapshots** — Don't guess page state. Read the HTML after each command.
2. **Use `snapshot` liberally** — When unsure about current state, take a snapshot.
3. **Wait between commands** — Give the browser time to settle. The built-in waits handle most cases, but dynamic SPAs may need extra time via `js` waits.
4. **Use `js` for complex interactions** — When CSS selectors aren't enough, JavaScript can handle shadow DOM, iframes, scrolling, and dynamic content.
5. **Check status** — Always check `result.txt` status before proceeding. An error means the page state may not have changed.
