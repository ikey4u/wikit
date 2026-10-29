# Wikit interface design

## Purpose and references

Wikit is a native dictionary for looking up, reading and revisiting words.
The interface puts one task first: get from a word to its meaning with as little
navigation and visual noise as possible. The shared NSH screen serves macOS,
iOS and Android; only dictionary HTML and previews use a WebView.

This redesign uses [Impeccable](https://github.com/pbakaus/impeccable)'s Operate
and Read principles, its new-work and craft-floor references, and the installed
macos-design skill's composition, interaction and visual guidance. Reviewed on
2026-10-29. The Impeccable context launcher was unavailable; product context was
read directly from the repository. Neither skill is an application dependency.

The `next` Electron implementation informed the compact navigation rail, direct
dictionary switching and reading history controls. These are implemented with
native NSH controls rather than restoring the retired Electron application.

## Composition

Desktop starts with a quiet 64-point icon sidebar, expandable to 208 points.
Every compact navigation item has an accessible label and tooltip. Search, Favorites, History and Notes sit
above a separated pair of Library and Preview tools. The wordmark uses native
text without a decorative icon tile. A light 56-point toolbar provides orientation
and work-in-progress feedback; platform window chrome remains native.

Below 800 points, four bottom destinations keep Search, Favorites, History and
Library within reach. Notes and Preview are available inside Library; Library
remains selected while either tool is open. Safe-area insets protect the navigation.

Content sits in a centered column up to 880 points wide, including its padding.
Definitions can expand to 1200 points to accommodate dictionary-authored layouts.
Horizontal padding is 40 points on desktop and 20 on smaller screens.

## Lookup and reading

Without a dictionary, show a focused setup invitation and one Add dictionary
action. Hide the unusable search field and dictionary filters. With dictionaries,
show the search field first, then a searchable native dictionary selector.
Back and forward controls revisit up to 50 definitions in the current reading
session without repeating the network request. Search stays disabled until a query and dictionary
are available. Return submits the query; the toolbar indicates loading.

Before a search, show up to five actual recent words for easy continuation. Never
fabricate definitions, history, sources or counts. Results pair a 32-point word
heading with dictionary metadata and Save word. A zero-result search has recovery
instructions and a route to dictionary management. The definition's original HTML,
CSS, scripts and media remain intact; dark mode does not recolor third-party HTML.

## Collections and tools

Saved words and history use whole-row actions: a word, its dictionary and a quiet
lookup icon. Empty collections explain how to populate them and offer an action
that accounts for whether any dictionaries exist. Hide Clear history when empty.

Library separates importing a .wikit file from connecting a server. Each mode
shows its relevant field, instructions and one submit action. Desktop file import
also exposes the native file chooser. Existing sources use separators rather than
nested cards. Refresh appears with existing sources. Appearance remains in Library.
Notes scroll independently of their save action. Preview remains available for
HTML authors. Internal data paths are omitted from the product interface.

## Visual system

Light: near-white #fbfcfa reading ground, #f0f3ee sidebar, #222d27 text,
#637168 secondary text, #dfe5dd dividers. Dark: #1d2221 ground, #232a26 sidebar,
#272e2b surfaces, #edf1ed text, #aebbb2 secondary text, #39463d dividers.
Forest green #33644d is reserved for primary actions and active controls;
selection surfaces are #e0e9de (light) and #344b3c (dark).

Use native system typography: 13–15-point controls/body, 18-point list words,
24–32-point headings, and readable 20–24-point supporting line heights. Native
Canvas supplies consistent monoline icons. Controls have restrained 6–7-point
radii; list items are divided by thin rules. Mobile actions retain at least
44-point targets and bottom navigation uses 60-point targets. Avoid decorative
cards, gradients, shadows, marketing panels and fake example data.

## Language and keyboard

Retain English defaults across the three clients. Dictionary content and authored
notes retain their language. Command/Control + 1–6 switches pages; Command/Control
+ S saves notes; Command/Control + F returns to and focuses search; Alt + Left/Right
revisits definitions. Return submits search or source entry. Success feedback clears
after 3.5 seconds; errors persist until another action starts.

## Verification boundaries

Run NSH checks for all three entries and headless interaction tests for navigation,
320/390-point widths, window resizing, Chinese system locale with English copy,
dark mode, empty collections, onboarding and import-mode switching.

The headless runner supplies no app data directory: it exercises UI interaction
but cannot certify persisted notes, actual dictionary lookup or import. Native
visual verification was attempted on Linux; the host build requires unavailable
GTK4 >= 4.14 and Graphene system libraries. Native macOS/iOS/Android captures and
real-device verification remain outstanding. No screenshots are claimed from
headless tests. Mobile system file picking remains unavailable in NSH; URLs and
accessible sandbox paths remain supported.
