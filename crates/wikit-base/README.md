# wikit-base

Reusable Rust parsing, indexing and lookup for `.wikit` and `.mdx` dictionaries.
The default feature set needs no GUI runtime, database, HTTP server, or application
configuration. This is the library requested in [issue #17](https://github.com/ikey4u/wikit/issues/17).

```rust,no_run
use wikit_base::Dictionary;

let dictionary = Dictionary::open("dictionary.wikit")?;
for (word, html) in dictionary.lookup("hello")? {
    println!("{word}: {html}");
}
# Ok::<(), wikit_base::WikitError>(())
```

`Dictionary::open` accepts explicit Wikit or MDX paths. Wikit keeps a reusable FST
index and reads definition records on demand. MDX is parsed into memory once;
exact and prefix lookup is then available. Wikit lookup returns the exact entry
first, followed by nearby spellings. An unknown word returns an empty vector.
HTML is returned intact; the caller chooses how to render it. MDX assets and
MDD archives are not automatically imported by this API.

The lower-level `parse_mdx`, `MDXDict`, `ParseOption`, and `LocalDictionary` APIs
remain public for format conversion and metadata inspection. `LocalDictionary::create`
builds a Wikit file from a text or MDX source and its adjacent TOML profile.
See [resource configuration](../../docs/resources.md) for CSS and JavaScript.

Optional `database` enables legacy PostgreSQL export; `preview` enables the legacy
source preview backend. Neither is required by the public dictionary API.

Until the alpha crates are published to crates.io, use a path dependency or a
Git dependency pinned to the alpha tag:

```toml
[dependencies]
wikit-base = { git = "https://github.com/ikey4u/wikit", tag = "v0.6.0-alpha.1" }
```

Run `mise exec -- cargo run -p wikit-base --example lookup -- dictionary.wikit hello`.
