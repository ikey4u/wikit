# Wikit native clients

The macOS, iOS and Android applications use [Native Shell](https://github.com/longbridge/nsh).
`shared/App.tsx` renders NSH native views and inputs. `native/` owns dictionary loading,
network requests, import, history, favorites, notes and settings in Rust and reuses
`wikit-core`'s dictionary reader and FST index. Only dictionary definitions and the HTML
preview use a platform WebView; there is no Tauri bridge or local HTTP server.

```
client/
  desktop/    macOS app configuration and entry
  ios/        iOS app configuration and entry
  android/    Android app configuration and entry
  shared/     shared native TSX screens
  native/     shared Rust module (wikit/native)
  scripts/    run, check, test and package commands
```

## Toolchain

Rust stable (edition 2024), Git and NSH from the **same revision** as `Cargo.toml`:

```sh
cargo install --git https://github.com/longbridge/nsh \
  --rev e40afd54cd0ca18707bc94ec836bd4eb4b977a75 --locked nsh-cli
```

macOS requires macOS 14+ and Xcode command line tools. iOS requires a Mac with
Xcode, an installed simulator or a development device, and iOS 15+. Android requires
JDK, Android SDK/NDK and a running emulator or connected device (Android 9/API 28+).
See the upstream [documentation](https://nsh.dev) for platform toolchain setup.

## Run and package

From the repository root:

```sh
client/scripts/wikit.sh run desktop
client/scripts/wikit.sh run ios
client/scripts/wikit.sh run android

client/scripts/wikit.sh package desktop
client/scripts/wikit.sh package ios
client/scripts/wikit.sh package android

client/scripts/wikit.sh check desktop
client/scripts/wikit.sh check ios
client/scripts/wikit.sh check android
client/scripts/wikit.sh test
nsh test client/desktop   # headless phone/desktop screen interactions
```

These commands use each platform's `nsh.json`; NSH generates and builds native hosts
and application packages in that entry's `.nsh/` directory. Platform signing and
release distribution use NSH's packaging options and the developer's own identities.
For iOS devices and App Store distribution configure signing on the build Mac.
The three configurations use the existing app identity `com.zhqli.wikit`.

`nsh check` verifies the host-side Rust library and shared TSX, including the
generated native bindings. It does not cross-compile or link the iOS/Android/macOS
application. Use the platform run/package commands on a machine with the required
SDK to verify native rendering, resource loading and device integration.
A source-only `cargo install nsh-cli` does not install the Android Java runtime.
Build the matching NSH checkout with `cargo xtask android --release --abi arm64-v8a`
and set `NSH_RUNTIME` to that checkout's `target` directory before Android packaging.
The pinned NSH revision also needs `patches/nsh-android-tokens.patch` applied to
its checkout before Android builds: the Android command encoder omitted the new
`Tokens` variant. The patch sends this control-only update through the existing
no-op branch; the core still resolves the app's styled token bindings.
The test-release workflow applies the patch and builds the runtime automatically.
The Native clients workflow runs the Rust tests, three entry checks and headless
UI tests on Linux; platform package signing is not part of that workflow.

## Dictionaries and data

In **词典**, connect a Wikit server by its base URL, such as `https://dict.example.com`.
The server must expose `/wikit/list`, `/wikit/query`, `/wikit/style` and `/wikit/script`.
Base path prefixes, URL-encoded Unicode/search characters and HTTP basic credentials
in a source URL are supported. Credentials are hidden from the source list.

For offline use enter an absolute `.wikit` path or an HTTP(S) download URL and choose
**导入到设备**. The validated file is copied into the app's private data directory.
macOS also provides a native file chooser. NSH's file-dialog API does not currently
support iOS or Android; these entries support URL and accessible sandbox-path imports.
A native mobile document picker is not implemented in this revision.
Dictionary HTML, CSS and JavaScript are preserved. Relative resources use the
original dictionary directory/server as their base; importing a standalone file
does not copy an adjacent external resource directory.

The host supplies its app-specific data directory to Rust. `client.json` stores sources,
the selected dictionary, the last 200 searches, favorites, notes and appearance.
Writes use an atomic temporary file in the same directory. On first use the app
copies sources from the existing `[cltcfg].uris` if available; it leaves `wikit.toml`
and server settings intact. Imported files live in `dictionaries/`.
Removing a source removes its history/favorites but preserves those of other sources.

**预览** displays a local HTML file or a directory's `index.html`; use **刷新预览**
after editing it, or enable **自动刷新** for a 1.5-second preview reload. It does not start the legacy desktop preview HTTP/WebSocket server.

The legacy `desktop/` and `android/` trees remain for reference and existing release
users; new GUI development and builds use `client/`. The CLI's workspace is separate,
and server/database/legacy preview dependencies are default-enabled optional features
of `wikit-core`. The new client disables those features for mobile builds.
