# Wikit server

```sh
mise run server -- -d dictionary.wikit -d another.mdx --listen 127.0.0.1:8888
```

The standalone `wikit-server` loads explicit dictionary paths once and shares
metadata/indexes across requests. No Rocket.toml or GUI configuration is needed.
The default address is loopback. Use `--listen 0.0.0.0:8888` when you intend to make
it reachable remotely, and place authentication/TLS at a reverse proxy if needed.
The server itself has no user/account system.

| Endpoint | Result |
| --- | --- |
| `/health` | Readiness response |
| `/wikit/list` | `[{"id": "…", "name": "…"}]` |
| `/wikit/query?dictname=ID&word=WORD` | JSON word/HTML pairs; `[]` if no match |
| `/wikit/style?dictname=ID` | Ordered dictionary CSS |
| `/wikit/script?dictname=ID` | Ordered dictionary JavaScript |
| `/wikit/base?dictname=ID` | Relative media root for that dictionary |
| `/wikit/assets/ID/path` | Supported media, font, CSS, JS and INI resources |

URL-encode query values. Unknown dictionary IDs return 404; empty/oversized queries
return 400. Querying does not depend on first calling `/wikit/list`. Blocking
format access runs on a worker pool. Ctrl+C shuts down the service gracefully.

Keep dictionary assets in a dedicated directory alongside the dictionary.
Asset paths cannot escape that directory, including through symlinks. Hidden
files and non-asset types (such as TOML/config files) are not served.
Raw MDX files expose entries but do not automatically load companion CSS/MDD
archives; convert to `.wikit` with a profile when you need bundled CSS and JS.

Native clients connect using the base URL; TUI clients use `--server URL`.
The library's `wikit_server::router(Catalog)` can be nested under a reverse-proxy
prefix or embedded in another Axum application.
