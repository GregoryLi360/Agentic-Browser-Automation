#!/usr/bin/env python3
"""Selenium browser agent — keeps one session open, accepts commands via a file.

Copies your browser profile to a temp dir so the browser can stay open.
Watches commands.txt for instructions, executes them, saves HTML snapshots to data/.

Commands (one per line in commands.txt):
  navigate <url>              — go to a URL
  click <css_selector>        — click an element
  type <css_selector> <text>  — type into an input
  select <css_selector> <value> — select a dropdown value
  snapshot                    — force-save current page HTML
  js <code>                   — execute arbitrary JS
"""

import argparse
import glob
import json
import os
import shutil
import tempfile
import time
from datetime import datetime

from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(BASE_DIR, "data")
CMD_FILE = os.path.join(BASE_DIR, "commands.txt")
RESULT_FILE = os.path.join(BASE_DIR, "result.txt")
POLL_INTERVAL = 1  # check for commands every second

# Profile search paths by browser and platform
_PROFILE_GLOBS = {
    "firefox": [
        "~/Library/Application Support/Firefox/Profiles/*.default-release",  # macOS
        "~/.mozilla/firefox/*.default-release",                              # Linux
        os.path.expandvars("%APPDATA%/Mozilla/Firefox/Profiles/*.default-release"),  # Windows
    ],
    "chrome": [
        "~/Library/Application Support/Google/Chrome",      # macOS
        "~/.config/google-chrome",                          # Linux
        os.path.expandvars("%LOCALAPPDATA%/Google/Chrome/User Data"),  # Windows
    ],
}


def find_default_profile(browser):
    """Auto-detect the default browser profile."""
    for pattern in _PROFILE_GLOBS.get(browser, []):
        expanded = os.path.expanduser(pattern)
        if "*" in expanded:
            matches = glob.glob(expanded)
            if matches:
                return matches[0]
        elif os.path.isdir(expanded):
            return expanded
    return None


def copy_profile(profile_path, browser):
    """Copy browser profile to a temp dir to avoid locking the original."""
    tmp_dir = tempfile.mkdtemp(prefix=f"{browser}_profile_")
    src = os.path.expanduser(profile_path)
    dst = os.path.join(tmp_dir, "profile")
    ignore = shutil.ignore_patterns(
        "lock", ".parentlock", "parent.lock",  # Firefox
        "lockfile", "SingletonLock", "SingletonCookie", "SingletonSocket",  # Chrome
    )
    shutil.copytree(src, dst, ignore=ignore)
    return dst, tmp_dir


def save_snapshot(html, label="page"):
    """Save an HTML snapshot. Returns the path."""
    os.makedirs(DATA_DIR, exist_ok=True)
    ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    path = os.path.join(DATA_DIR, f"{ts}_{label}.html")
    with open(path, "w", encoding="utf-8") as f:
        f.write(html)
    return path


def write_result(status, message, snapshot_path=None):
    """Write command result for the caller to read."""
    result = {"status": status, "message": message, "time": datetime.now().isoformat()}
    if snapshot_path:
        result["snapshot"] = snapshot_path
    with open(RESULT_FILE, "w") as f:
        json.dump(result, f)
    print(f"  -> {status}: {message}")


def execute_command(driver, line):
    """Parse and execute a single command."""
    parts = line.strip().split(None, 1)
    if not parts:
        return

    cmd = parts[0].lower()
    arg = parts[1] if len(parts) > 1 else ""

    try:
        if cmd == "navigate":
            driver.get(arg)
            WebDriverWait(driver, 15).until(
                EC.presence_of_element_located((By.TAG_NAME, "body"))
            )
            time.sleep(2)
            path = save_snapshot(driver.page_source, "nav")
            write_result("ok", f"Navigated to {arg}", path)

        elif cmd == "click":
            el = WebDriverWait(driver, 10).until(
                EC.element_to_be_clickable((By.CSS_SELECTOR, arg))
            )
            el.click()
            time.sleep(2)
            path = save_snapshot(driver.page_source, "click")
            write_result("ok", f"Clicked {arg}", path)

        elif cmd == "type":
            selector, text = arg.split(None, 1)
            el = WebDriverWait(driver, 10).until(
                EC.presence_of_element_located((By.CSS_SELECTOR, selector))
            )
            el.clear()
            el.send_keys(text)
            time.sleep(1)
            path = save_snapshot(driver.page_source, "type")
            write_result("ok", f"Typed into {selector}", path)

        elif cmd == "select":
            selector, value = arg.split(None, 1)
            from selenium.webdriver.support.ui import Select
            el = WebDriverWait(driver, 10).until(
                EC.presence_of_element_located((By.CSS_SELECTOR, selector))
            )
            Select(el).select_by_value(value)
            time.sleep(1)
            path = save_snapshot(driver.page_source, "select")
            write_result("ok", f"Selected {value} in {selector}", path)

        elif cmd == "snapshot":
            path = save_snapshot(driver.page_source, "snap")
            write_result("ok", "Snapshot saved", path)

        elif cmd == "js":
            result = driver.execute_script(arg)
            time.sleep(1)
            path = save_snapshot(driver.page_source, "js")
            write_result("ok", f"JS result: {result}", path)

        else:
            write_result("error", f"Unknown command: {cmd}")

    except Exception as e:
        write_result("error", f"{cmd} failed: {e}")


def create_driver(browser, profile_path, headless, no_profile):
    """Create a Selenium WebDriver for the given browser."""
    tmp_dir = None

    if browser == "firefox":
        from selenium.webdriver.firefox.options import Options
        opts = Options()
        if headless:
            opts.add_argument("--headless")
        if not no_profile:
            profile_path = profile_path or os.environ.get("FIREFOX_PROFILE") or find_default_profile("firefox")
            if profile_path is None:
                raise SystemExit("Could not find a Firefox profile. Use --profile, set FIREFOX_PROFILE, or use --no-profile.")
            print(f"Copying Firefox profile from {profile_path}...")
            profile_copy, tmp_dir = copy_profile(profile_path, "firefox")
            opts.add_argument("-profile")
            opts.add_argument(profile_copy)
        driver = webdriver.Firefox(options=opts)

    elif browser == "chrome":
        from selenium.webdriver.chrome.options import Options
        opts = Options()
        if headless:
            opts.add_argument("--headless=new")
        if not no_profile:
            profile_path = profile_path or os.environ.get("CHROME_PROFILE") or find_default_profile("chrome")
            if profile_path is None:
                raise SystemExit("Could not find a Chrome profile. Use --profile, set CHROME_PROFILE, or use --no-profile.")
            print(f"Copying Chrome profile from {profile_path}...")
            profile_copy, tmp_dir = copy_profile(profile_path, "chrome")
            opts.add_argument(f"--user-data-dir={profile_copy}")
        driver = webdriver.Chrome(options=opts)

    else:
        raise SystemExit(f"Unsupported browser: {browser}. Use 'firefox' or 'chrome'.")

    if not no_profile and tmp_dir is None:
        print("Using fresh profile (no cookies).")

    return driver, tmp_dir


def main():
    parser = argparse.ArgumentParser(description="Selenium browser agent with command file interface")
    parser.add_argument("url", help="Starting URL to open")
    parser.add_argument("-b", "--browser", default="firefox", choices=["firefox", "chrome"],
                        help="Browser to use (default: firefox)")
    parser.add_argument("--profile", default=None,
                        help="Browser profile path (default: auto-detect)")
    parser.add_argument("--no-profile", action="store_true",
                        help="Use a fresh profile (no cookies/auth)")
    parser.add_argument("--headless", action="store_true",
                        help="Run in headless mode (no visible window)")
    args = parser.parse_args()

    driver, tmp_dir = create_driver(args.browser, args.profile, args.headless, args.no_profile)
    driver.get(args.url)
    time.sleep(3)
    path = save_snapshot(driver.page_source, "init")
    print(f"Opened {args.url} — saved {path}")
    print(f"Watching {CMD_FILE} for commands...")

    # Clear any stale commands
    if os.path.exists(CMD_FILE):
        os.remove(CMD_FILE)

    try:
        while True:
            if os.path.exists(CMD_FILE):
                with open(CMD_FILE, "r") as f:
                    lines = f.readlines()
                os.remove(CMD_FILE)

                for line in lines:
                    line = line.strip()
                    if line and not line.startswith("#"):
                        print(f"[{datetime.now().strftime('%H:%M:%S')}] Executing: {line}")
                        execute_command(driver, line)

            time.sleep(POLL_INTERVAL)
    except KeyboardInterrupt:
        print("\nStopping.")
    finally:
        driver.quit()
        if tmp_dir:
            shutil.rmtree(tmp_dir, ignore_errors=True)


if __name__ == "__main__":
    main()
