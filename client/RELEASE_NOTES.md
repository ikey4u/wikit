Wikit's redesigned native dictionary interface is now available for testing.

## What's new

- A quieter forest-green visual system with refined light and dark themes.
- A focused lookup and reading layout, with recent words ready to revisit.
- A compact desktop sidebar and four primary destinations in mobile bottom navigation.
- Clear first-use guidance, helpful empty collections, and recovery for searches without results.
- Separate flows for importing a dictionary file and connecting a Wikit server.
- English interface defaults across macOS, iOS, and Android. Dictionary content and personal notes keep their original language.

Local and remote lookup, favorites, search history, study notes, appearance settings, and HTML preview remain available.

## Downloads

- **Android:** Android 9 or later, ARM64. Download and install the `.apk` file.
- **macOS:** macOS 14 or later, Apple Silicon. Download the `.zip`, extract it, and open `Wikit.app`.
- **Checksums:** Use `SHA256SUMS.txt` to verify the SHA-256 hashes of the downloads.

## Installation notes

The Android package uses a test signing key and cannot replace an installation signed with a different key. The macOS application is ad-hoc signed and is not notarized by Apple; the first launch may require allowing it in **System Settings > Privacy & Security**.

## Known limitations and verification

- Mobile system file picking is not available yet. Import using a download URL or an accessible sandbox path.
- Importing a standalone `.wikit` file does not copy adjacent external media directories.
- Dictionary-provided HTML retains its original styling, including in dark mode.
- iOS source checks pass, but an iOS installation package is not included in this release.
- Shared UI checks pass for all three clients, and five headless interaction tests pass. Headless tests do not certify native rendering, persisted data, or real-device behavior. Native visual and device acceptance testing remains outstanding.

This is a prerelease for testing. Please report issues through GitHub Issues. The release tag identifies the exact source commit used to build these packages.
