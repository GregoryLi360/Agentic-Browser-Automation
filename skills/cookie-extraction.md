# Skill: Cookie Extraction

How to use `cookies.py` and `fetch.py` to extract browser cookies and make authenticated HTTP requests without Selenium.

## When to Use This vs browse.py

| Use case | Tool |
|---|---|
| Need to interact with a web page (click, type, navigate) | `browse.py` |
| Need to call an API or download a file with auth | `fetch.py` or `cookies.py` |
| Need cookies for another tool/script | `cookies.py` |

## fetch.py — Quick Authenticated HTTP

```bash
# Fetch a URL with Firefox cookies
python3 fetch.py https://api.example.com/data

# Use Chrome cookies
python3 fetch.py https://api.example.com/data -b chrome

# Save to file
python3 fetch.py https://example.com/export.csv -o export.csv
```

```python
from fetch import fetch

# Returns a requests.Response object
resp = fetch("https://api.example.com/data", browser="firefox")
data = resp.json()

# With Chrome
resp = fetch("https://api.example.com/data", browser="chrome")
```

## cookies.py — Direct Cookie Extraction

For lower-level access to cookies:

```python
import cookies

class SimpleLogger:
    def debug(self, msg): pass
    def info(self, msg): print(msg)
    def warning(self, msg): print(f"WARN: {msg}")
    def error(self, msg): print(f"ERROR: {msg}")
    def stdout(self, msg): print(msg)

logger = SimpleLogger()

# Extract all cookies from Firefox
jar = cookies.extract_cookies_from_browser("firefox", profile=None, logger=logger)

# Extract from Chrome
jar = cookies.extract_cookies_from_browser("chrome", profile=None, logger=logger)

# Filter cookies for a specific domain
for cookie in jar:
    if "example.com" in cookie.domain:
        print(f"{cookie.name}={cookie.value}")
```

## Supported Browsers

| Browser | Name to use |
|---|---|
| Firefox | `firefox` |
| Chrome | `chrome` |
| Chromium | `chromium` |
| Brave | `brave` |
| Edge | `edge` |
| Opera | `opera` |
| Vivaldi | `vivaldi` |
| Safari | `safari` (macOS only) |

## How It Works

- **Firefox**: Reads `cookies.sqlite` database directly
- **Chrome/Chromium**: Reads encrypted cookie database, decrypts using OS keychain (macOS), keyring (Linux), or DPAPI (Windows)
- **Safari**: Parses binary cookie format from `~/Library/Cookies/Cookies.binarycookies`

## Common Issues

- **Browser must be closed** (or at least the profile must not be locked) for cookie extraction to work reliably
- **macOS Keychain prompt**: Chrome cookie decryption may trigger a keychain access prompt
- **Linux keyring**: Requires `secretstorage` package for GNOME keyring, or KWallet for KDE
