# Skill: Troubleshooting

Common issues and solutions when working with this repo.

## Setup Issues

### geckodriver / chromedriver not found
```
selenium.common.exceptions.WebDriverException: Message: 'geckodriver' executable needs to be in PATH
```
**Fix**: Install the driver for your browser:
```bash
# macOS
brew install geckodriver    # Firefox
brew install chromedriver   # Chrome

# Or download directly:
# Firefox: https://github.com/mozilla/geckodriver/releases
# Chrome: https://googlechromelabs.github.io/chrome-for-testing/
```

### Profile not found
```
Could not find a Firefox profile. Use --profile, set FIREFOX_PROFILE, or use --no-profile.
```
**Fix**: Either set the env var, pass `--profile`, or use `--no-profile` for a fresh session:
```bash
# Find your Firefox profile
ls ~/Library/Application\ Support/Firefox/Profiles/

# Set it
export FIREFOX_PROFILE="$HOME/Library/Application Support/Firefox/Profiles/xxxxxxxx.default-release"

# Or skip profile entirely
python3 browse.py https://example.com --no-profile
```

### Browser already running (profile locked)
The script copies your profile to a temp directory, so this usually isn't an issue. But if you see lock errors:
```bash
# Make sure you're not running another instance of browse.py
ps aux | grep browse.py
```

## Runtime Issues

### Command not executing
- Check that `commands.txt` is being written to the repo root (same dir as `browse.py`)
- The file is deleted after reading — if it persists, the script may have crashed
- Check the terminal where `browse.py` is running for errors

### Element not found / not clickable
1. Take a snapshot: `echo "snapshot" > commands.txt`
2. Read the snapshot HTML to find the correct selector
3. The element may be in an iframe — use `js` to switch:
   ```
   js document.querySelector('iframe').contentDocument.querySelector('button').click()
   ```
4. The element may need scrolling:
   ```
   js document.querySelector('.target').scrollIntoView()
   ```

### Page didn't load / timeout
- Check if the site requires specific cookies or auth that your profile doesn't have
- Some sites block Selenium — look for bot detection
- Try with `--headless` disabled to see what's happening visually

### Dynamic content not in snapshot
Single-page apps load content dynamically. After navigating, use JS to wait:
```
js return document.querySelector('.content')?.textContent?.length > 0
```
If it returns `false` or `None`, the content hasn't loaded yet. Wait and take another snapshot.

## cookies.py Issues

### ImportError: logger module
`cookies.py` imports from a `logger` module that doesn't exist in this repo. If you need to use `cookies.py` directly (not through `fetch.py`), provide a logger object with `debug`, `info`, `warning`, `error` methods. See `fetch.py:SimpleLogger` for a minimal implementation.

### macOS keychain prompt
Chrome cookie decryption triggers a keychain access dialog. Click "Allow" or "Always Allow" when prompted.

### Permission denied on cookie database
The browser may have the database locked. Close the browser or use the profile copy approach (which `browse.py` already does).
