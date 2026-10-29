#!/usr/bin/env python3
"""Check the single alpha version source and publish its Git tag."""
import argparse
import json
import re
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def version():
    return tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]


def check(tag=None):
    current = version()
    if not re.fullmatch(r"0\.6\.0-alpha\.[1-9][0-9]*", current):
        raise ValueError("Expected a SemVer alpha version: 0.6.0-alpha.N")
    if tag is not None and tag != f"v{current}":
        raise ValueError(f"Release tag must be v{current}, got {tag}")
    for platform in ("desktop", "ios", "android"):
        config = json.loads((ROOT / "client" / platform / "nsh.json").read_text())
        if config["version"] != current:
            raise ValueError(f"{platform} version differs from workspace version")
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]
    for name in ("wikit-base", "wikit-proto"):
        if workspace["dependencies"][name]["version"] != current:
            raise ValueError(f"{name} dependency version differs from workspace version")
    print(f"Validated alpha release: v{current}")
    return current


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("check", "publish"))
    parser.add_argument("--tag")
    args = parser.parse_args()
    current = check(args.tag)
    if args.action == "publish":
        def git(*arguments):
            return subprocess.check_output(["git", *arguments], cwd=ROOT, text=True).strip()
        if git("status", "--porcelain"):
            raise SystemExit("Commit all release changes before publishing.")
        tag = f"v{current}"
        # Existing tags are errors: never overwrite a published version.
        git("tag", "-a", tag, "-m", f"Wikit {current}")
        subprocess.run(["git", "push", "origin", tag], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
