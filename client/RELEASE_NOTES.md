Wikit 0.6.0 is now managed as a SemVer alpha release. This release reorganizes the project around reusable libraries and adds terminal and server applications.

## Highlights

- One Cargo workspace and a mise-managed toolchain for development, checks and packaging.
- A public `wikit-base` library for Wikit/MDX parsing, reading and lookup, with an independent default feature set and examples.
- Ordered multi-file CSS and JavaScript support: single filenames, comma-separated lists and TOML arrays. Missing resources now produce actionable errors.
- A standalone `wikit-server` with compatible dictionary endpoints and per-dictionary media resources.
- A UTF-8 `wikit-tui` for interactive terminals and SSH, plus one-shot text/JSON output for Vim, Emacs and pipes.
- A compact, expandable native desktop sidebar, searchable dictionary selection, search shortcut, and backward/forward reading inspired by the Electron design on `next`.
- MDX writing/reading fixes and aligned UTF-16 key parsing, covered by round-trip tests.

## Downloads

- **Android ARM64:** Android 9+, `.apk`.
- **macOS Apple Silicon:** macOS 14+, native app `.zip`.
- **Linux x64 and macOS ARM64 tools:** `.tar.gz` containing `wikit`, `wikit-server`, `wikit-tui` and usage guides.
- **SHA256SUMS.txt:** SHA-256 hashes for all four download packages.

Example: `wikit-server -d dictionary.wikit`, then `wikit-tui --server http://127.0.0.1:8888 hello`.

## Migration and limitations

- Development tools are installed with `mise install`. Use the root Cargo workspace and scripts directory. The old root Android/Tauri implementations and Rocket.toml have been removed.
- Use `wikit-server` instead of `wikit server --start`. Existing native app identity and client data format are preserved.
- Previous `-test` tags remain historical; new releases use `0.6.0-alpha.N`.
- Android uses a test signing key. macOS is ad-hoc signed and not notarized; installation may require allowing it in Privacy & Security.
- iOS source checks are included, but an iOS installation package is not distributed.
- Mobile system file picking remains unavailable. Standalone dictionary imports do not copy adjacent external media directories.
- TUI converts HTML to terminal text; browser scripts, images and audio are not executed. Raw MDX reading does not automatically import MDD assets or companion styles.
- Library packages are usable from the repository; this release does not publish them to crates.io.

This is an alpha for testing. Automated checks do not replace native visual and real-device acceptance testing. Please report problems through GitHub Issues.
