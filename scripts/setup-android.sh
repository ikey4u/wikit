#!/usr/bin/env bash
set -euo pipefail
# The mise android-sdk backend supplies sdkmanager and ANDROID_HOME.
: "${ANDROID_HOME:?Run this script with mise run setup:android}"
python - <<'LICENSES'
import os, subprocess
subprocess.run(["sdkmanager", "--sdk_root=" + os.environ["ANDROID_HOME"], "--licenses"], input="y\n" * 100, text=True, check=True)
LICENSES
sdkmanager --sdk_root="$ANDROID_HOME" "platforms;android-35" "build-tools;35.0.1" "ndk;27.2.12479018"
rustup target add aarch64-linux-android
if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "ANDROID_HOME=$ANDROID_HOME" >> "$GITHUB_ENV"
  echo "ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.2.12479018" >> "$GITHUB_ENV"
fi
