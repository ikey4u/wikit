#!/usr/bin/env python3
"""Package native apps and portable command-line tools for the release matrix."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[1]
platform = sys.argv[1]
tag = os.environ.get("GITHUB_REF_NAME", "local-alpha")
output = Path(os.environ.get("RUNNER_TEMP", str(ROOT / "inner"))) / "wikit-package"
artifacts = ROOT / "artifacts"
artifacts.mkdir(exist_ok=True)


def run(*args):
    subprocess.run([str(arg) for arg in args], cwd=ROOT, check=True)


if platform == "android-arm64":
    run("nsh", "package", "client/android", "--target", "android", "--abi", "arm64-v8a", "--out", output)
    apk = artifacts / f"wikit-{tag}-android-arm64.apk"
    shutil.copy2(output / "Wikit.apk", apk)
    build_tools = Path(os.environ["ANDROID_HOME"]) / "build-tools/35.0.1"
    run(build_tools / "apksigner", "verify", "--verbose", apk)
    run(build_tools / "aapt", "dump", "badging", apk)
    tools_platform = "linux-x64"
elif platform == "macos-arm64":
    import platform as system
    if system.machine() != "arm64":
        raise SystemExit("The macOS build requires an Apple Silicon runner")
    run("nsh", "package", "client/desktop", "--target", "macos", "--out", output)
    run("codesign", "--verify", "--deep", "--strict", output / "Wikit.app")
    run("ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", output / "Wikit.app", artifacts / f"wikit-{tag}-macos-arm64.zip")
    tools_platform = "macos-arm64"
else:
    raise SystemExit("Expected android-arm64 or macos-arm64")

run("cargo", "build", "--release", "--locked", "-p", "wikit", "-p", "wikit-server", "-p", "wikit-tui")
with tarfile.open(artifacts / f"wikit-{tag}-{tools_platform}-tools.tar.gz", "w:gz") as archive:
    for executable in ("wikit", "wikit-server", "wikit-tui"):
        path = ROOT / "target/release" / executable
        run(path, "--version")
        archive.add(path, arcname=executable)
    archive.add(ROOT / "LICENSE", arcname="LICENSE")
    for source, name in [("server/README.md", "SERVER.md"), ("client/tui/README.md", "TUI.md"), ("README.md", "README.md")]:
        archive.add(ROOT / source, arcname=name)
