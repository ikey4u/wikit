#!/usr/bin/env python3
"""Verify the alpha artifact set, create checksums and publish English notes."""
import hashlib
import os
from pathlib import Path
import subprocess
from release import check

root = Path(__file__).resolve().parents[1]
tag = os.environ["GITHUB_REF_NAME"]
check(tag)
artifacts = root / "artifacts"
expected = [f"wikit-{tag}-android-arm64.apk", f"wikit-{tag}-macos-arm64.zip", f"wikit-{tag}-linux-x64-tools.tar.gz", f"wikit-{tag}-macos-arm64-tools.tar.gz"]
lines = []
for filename in expected:
    path = artifacts / filename
    if not path.is_file() or path.stat().st_size == 0:
        raise SystemExit(f"Missing or empty release artifact: {filename}")
    with path.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    lines.append(f"{digest}  {filename}\n")
(artifacts / "SHA256SUMS.txt").write_text("".join(lines))
subprocess.run(["gh", "release", "create", tag, *[str(artifacts / name) for name in expected], str(artifacts / "SHA256SUMS.txt"), "--repo", os.environ["GITHUB_REPOSITORY"], "--verify-tag", "--prerelease", "--title", f"Wikit {tag.removeprefix('v')}", "--notes-file", str(root / "client/RELEASE_NOTES.md")], check=True)
