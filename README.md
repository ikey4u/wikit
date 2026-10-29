<p align="center"><img src="client/nsh/icon.png" width="96" alt="Wikit" /></p>

# Wikit

A dictionary toolkit for reading, building and serving dictionaries. Wikit combines
native macOS/iOS/Android clients, a UTF-8 terminal client, an HTTP server and reusable
Rust libraries in one Cargo workspace.

Wikit 0.6.0 is an **alpha**. Releases follow SemVer (`0.6.0-alpha.1`,
`0.6.0-alpha.2`, …), with matching `v`-prefixed Git tags. Downloads and release notes
are on [GitHub Releases](https://github.com/ikey4u/wikit/releases).

## Develop

Install [mise](https://mise.jdx.dev/getting-started.html), then:

```sh
mise trust
mise install
mise run check
mise run test
mise run check:native
```

`mise.toml` pins Rust, Python, Java, Android SDK tools, GitHub CLI and the exact NSH CLI revision.
`mise.lock` records downloadable tool metadata for Linux x64 and macOS ARM64.
Run tasks through mise, or prefix individual commands with `mise exec --`.
Platform SDKs still require their platform: macOS/iOS builds need Xcode on a Mac;
`mise run setup:android` installs the Android SDK/NDK using the mise-managed SDK.
A C/C++ linker and normal OS build libraries are host prerequisites.

## Use

```sh
# Build a dictionary (source can be .txt or .mdx)
mise run cli -- dict --create dictionary.txt --output dictionary.wikit

# Interactive terminal dictionary, including over SSH
mise run tui -- --dictionary dictionary.wikit

# One-shot text output for Vim, Emacs or pipes; --json returns word/HTML pairs
mise run tui -- --dictionary dictionary.wikit hello

# Serve one or more local dictionaries
mise run server -- --dictionary dictionary.wikit --listen 127.0.0.1:8888

# Query that server from a terminal
mise run tui -- --server http://127.0.0.1:8888 hello

# Native desktop on macOS
mise run desktop
```

Native apps can connect to the server's base URL in Library. See the
[native client guide](client/README.md), [TUI guide](client/tui/README.md),
[server guide](server/README.md), and [CSS/JS resource guide](docs/resources.md).

## Repository

| Directory | Purpose |
| --- | --- |
| `crates/wikit-base` | MDX/Wikit parsing, reusable index and lookup API |
| `crates/wikit-proto` | Shared serializable protocol types |
| `crates/wikit-client` | Native bridge and persisted client business logic |
| `client/nsh` | Shared NSH native interface and assets |
| `client/desktop`, `client/ios`, `client/android` | Native platform entries |
| `client/tui` | Interactive and one-shot terminal dictionary |
| `server` | Standalone HTTP service |
| `tools/cli` | Dictionary creation and conversion CLI |
| `scripts` | All project maintenance/build/release scripts |
| `patches` | Patches for pinned third-party dependencies |

The root `Cargo.toml` and `Cargo.lock` manage every Rust package. NSH-generated
`.nsh` build projects and `target` directories are disposable and ignored.
The old root Android and Tauri implementations have been removed; Git history
retains them. `wikit server --start` is replaced by the dedicated `wikit-server`.

## Embed the library

See [wikit-base](crates/wikit-base/README.md). Its default API does not need a GUI,
server, database or global user configuration:

```rust,no_run
let dictionary = wikit_base::Dictionary::open("dictionary.wikit")?;
let entries = dictionary.lookup("hello")?;
# Ok::<(), wikit_base::WikitError>(())
```

## Release

Update the workspace version, internal dependency versions and all three NSH app
versions together. Increase app build numbers for installable updates. Then run
`mise run release:check`, commit the verified changes, push the branch, and run
`mise run release:publish`. It creates an immutable SemVer alpha tag; GitHub Actions
builds and publishes English release notes, native packages, terminal/server/CLI
archives and SHA-256 checksums. It never overwrites existing tags.

Library crates are prepared for external use but are not automatically uploaded
to crates.io. Use a pinned Git tag or a local path until registry publication.

[MIT License](LICENSE)
