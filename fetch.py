#!/usr/bin/env python3
"""Fetch any URL using cookies extracted from your browser."""

import argparse
import sys
import requests

import cookies


class SimpleLogger:
    def debug(self, msg): pass
    def info(self, msg): print(msg)
    def warning(self, msg): print(f"WARN: {msg}")
    def error(self, msg): print(f"ERROR: {msg}", file=sys.stderr)
    def stdout(self, msg): print(msg)


def fetch(url, browser="firefox"):
    """Fetch a URL using cookies from the given browser. Returns the Response."""
    logger = SimpleLogger()
    jar = cookies.extract_cookies_from_browser(browser, profile=None, logger=logger)

    session = requests.Session()
    for cookie in jar:
        session.cookies.set_cookie(cookie)

    resp = session.get(url)
    resp.raise_for_status()
    return resp


def main():
    parser = argparse.ArgumentParser(description="Fetch a URL with browser cookies")
    parser.add_argument("url", help="URL to fetch")
    parser.add_argument("-b", "--browser", default="firefox",
                        help="Browser to extract cookies from (default: firefox)")
    parser.add_argument("-o", "--output", help="Write response body to file instead of stdout")
    args = parser.parse_args()

    resp = fetch(args.url, args.browser)

    if args.output:
        with open(args.output, "wb") as f:
            f.write(resp.content)
        print(f"Saved to {args.output}")
    else:
        print(resp.text)


if __name__ == "__main__":
    main()
