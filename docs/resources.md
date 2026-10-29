# Dictionary CSS, JavaScript and media

Place `dictionary.toml` next to `dictionary.txt` or `dictionary.mdx` before creating
`dictionary.wikit`. CSS and JavaScript load relative to that source directory,
independently of the command's working directory. They are embedded in the Wikit
file in the configured order, so later CSS can override earlier rules and scripts
can depend on earlier libraries.

```toml
name = "LDOCE_5th"
css = ["LM5style.css", "LM5style_switch.css", "LM5style_show.css"]
js = ["jquery-3.2.1.min.js", "LM5Switch.js"]
```

Single filenames and the comma-separated syntax from issue #15 also work:

```toml
css = "LM5style.css, LM5style_switch.css, LM5style_show.css"
js = "jquery-3.2.1.min.js, LM5Switch.js"
```

Legacy `css = "@dictionary.css"` and inline CSS/JS remain supported. Empty lists
and omitted resources are valid. Missing referenced files and invalid TOML are
errors; they are never silently replaced by empty styles. Arrays explicitly mean
filenames and are preferred for filenames that contain commas.

Relative `url(...)` references in CSS files in subdirectories are rebased to the
source directory. Keep media, fonts, imported stylesheets and other external
resources alongside the deployed dictionary with their relative layout intact.
A standalone `.wikit` import does not bundle those external directories.

An `.ini` file has no universal browser or Wikit interpretation. If dictionary
JavaScript explicitly fetches an INI resource, retain it at the expected relative
path; otherwise its settings must be translated into that dictionary's JS/CSS.
Wikit does not emulate another dictionary application's proprietary INI behavior.

The HTTP server exposes assets beneath `wikit/assets/<dictionary-id>/` and the
`/wikit/base?dictname=<id>` endpoint tells new clients that relative base. Only
supported media, font, CSS, JS and INI files inside the dictionary directory are
served; hidden paths, unrelated file types and escaping symlinks are rejected.
