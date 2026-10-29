# Wikit native clients

The macOS, iOS and Android apps use the pinned Native Shell (NSH) revision.
`nsh/App.tsx` renders native views; `../crates/wikit-client` implements dictionary
loading, import, history, favorites, notes and settings. Only dictionary HTML and
HTML preview use a WebView.

From the repository root:

```sh
mise trust
mise install
mise run check:native
mise exec -- nsh test client/desktop
mise run desktop
mise exec -- scripts/wikit.sh run ios
mise exec -- scripts/wikit.sh run android
mise exec -- scripts/wikit.sh package desktop
```

macOS requires macOS 14+ and Xcode command-line tools. iOS requires a Mac with
Xcode and a simulator or configured signing for a device. Android requires Android
9+ on the device; run `mise run setup:android` for SDK 35 and NDK 27, and connect a
device/emulator. Export `ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.2.12479018` when
building outside GitHub Actions. `mise run build:android-runtime` prepares the
matching NSH runtime and prints the `NSH_RUNTIME` path to export before packaging.
The root `patches/nsh-android-tokens.patch` is applied idempotently by this task.

In **Library**, choose file import or server connection. A server base URL such
as `http://127.0.0.1:8888` uses `/wikit/list`, `query`, `style`, and `script`.
New servers also provide a per-dictionary media base; older servers retain the
legacy base URL fallback. URL prefixes, encoded queries and HTTP Basic credentials
remain supported. Credentials are not shown in the source list.

Offline imports accept `.wikit` download URLs or accessible absolute paths; macOS
also has a native file chooser. Mobile system document picking is not yet available.
Importing a standalone dictionary does not copy adjacent media directories.

Client data remains in the platform's private app directory, with the existing
`client.json` schema. First launch can reuse existing `[cltcfg].uris`. Repository
reorganization does not change app identity or the stored user data format.

Desktop takes cues from the `next` Electron interface: a compact expandable
navigation rail, focused search, searchable dictionary selection, and backward /
forward reading. `Cmd/Ctrl+F` returns to search; `Alt+Left/Right` visits cached
readings; `Cmd/Ctrl+1..6` navigates pages and `Cmd/Ctrl+S` saves notes. Phones retain
four bottom destinations and safe-area spacing. English is the default interface
language; dictionary content is preserved.

Source checks do not cross-compile or certify native rendering. Headless UI tests
lack an app data directory and cover interaction rather than persistence. Platform
packaging, signing and device acceptance are separate checks.
