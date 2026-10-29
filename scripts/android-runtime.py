#!/usr/bin/env python3
"""Build the NSH revision pinned in Cargo.toml, including its Android token patch."""
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1"], cwd=root))
package = next(p for p in metadata["packages"] if p["name"] == "nsh-core")
source = Path(package["manifest_path"]).parents[2]
patch = root / "patches/nsh-android-tokens.patch"
check = subprocess.run(["git", "-C", str(source), "apply", "--check", str(patch)], capture_output=True)
if check.returncode == 0:
    subprocess.run(["git", "-C", str(source), "apply", str(patch)], check=True)
else:
    subprocess.run(["git", "-C", str(source), "apply", "--reverse", "--check", str(patch)], check=True)
subprocess.run(["cargo", "run", "--manifest-path", str(source / "Cargo.toml"), "-p", "xtask", "--", "android", "--release", "--abi", "arm64-v8a"], check=True)
runtime = source / "target"
if not (runtime / "android/release/classes.dex").is_file():
    raise SystemExit("NSH Android runtime is missing classes.dex")
print(f"NSH_RUNTIME={runtime}")
if os.environ.get("GITHUB_ENV"):
    with open(os.environ["GITHUB_ENV"], "a") as output:
        output.write(f"NSH_RUNTIME={runtime}\n")
