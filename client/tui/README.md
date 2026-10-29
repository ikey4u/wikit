# Wikit TUI

A UTF-8 terminal dictionary for local sessions, SSH and editor integration
([issue #16](https://github.com/ikey4u/wikit/issues/16)). No graphical session is required.

```sh
mise run tui -- -d dictionary.wikit
mise run tui -- -d dictionary.mdx
mise run tui -- --server http://127.0.0.1:8888
```

Repeat `-d` or `--server` for multiple dictionaries; `--select N` chooses the initial
one (starting at 1). Tab/Shift+Tab cycles dictionaries. Type a word and press Enter;
use Up/Down for entries, PageUp/PageDown for definitions, `/` to edit the query,
F1 for help and Ctrl+C to quit. While reading, `q` and Escape also quit.
The terminal is restored on normal exit, errors and Rust panics. Queries run off
the input/render loop, so a slow server does not freeze keyboard handling.

Pass a positional word for one-shot output. This works without a TTY and does not
emit terminal control sequences:

```sh
wikit-tui -d dictionary.wikit hello
wikit-tui --server https://dictionary.example.com hello --json
ssh host wikit-tui -d /srv/dictionaries/english.wikit hello
```

In Vim, `:read !wikit-tui -d /path/dictionary.wikit hello` inserts a definition;
in Emacs, use `M-x shell-command` with the same command. Quote shell arguments
when passing text selected in an editor. `--json` prints word/HTML pairs for
programmatic integration; `--width 100` controls plain text wrapping.

HTML is converted to terminal paragraphs/lists rather than executed. Styles,
scripts and interactive dictionary widgets are not supported in a terminal.
Images/audio remain part of the original dictionary and can be viewed in the
native client. MDX reading uses the library's in-memory index; very large MDX files
may benefit from conversion to `.wikit` first.
