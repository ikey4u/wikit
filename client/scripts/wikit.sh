#!/usr/bin/env bash
set -euo pipefail
client_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
action="${1:-run}"
platform="${2:-desktop}"
if [[ $# -ge 2 ]]; then shift 2; else set --; fi
case "$platform" in
  desktop) target=macos ;;
  ios) target=ios ;;
  android) target=android ;;
  *) echo 'Platform must be desktop, ios or android' >&2; exit 2 ;;
esac
case "$action" in
  run|package) exec nsh "$action" "$client_dir/$platform" --target "$target" "$@" ;;
  check) exec nsh check "$client_dir/$platform" "$@" ;;
  test) exec cargo test --manifest-path "$client_dir/Cargo.toml" --locked -p wikit-client-native "$@" ;;
  *) echo 'Usage: wikit.sh [run|package|check|test] [desktop|ios|android] [options]' >&2; exit 2 ;;
esac
