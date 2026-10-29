# Wikit interface design

Wikit is a focused dictionary and reading utility. The word and its definition
are the main content; navigation and management controls should remain quiet.

## Reference

This redesign applies the layout, composition, interaction, and visual guidance
from [ceorkm/macos-design-skill](https://github.com/ceorkm/macos-design-skill),
reviewed on 2026-10-29. The skill is installed in the developer environment;
it is not an application dependency. Native NSH views remain the renderer.

## Composition

- A 224-point desktop sidebar anchors six stable destinations. Monoline icons,
  subtle selection, and history/favorite counts establish hierarchy.
- A 56-point content toolbar identifies the current destination. Search stays
  above the reading surface. No simulated traffic lights or web window chrome:
  the platform owns the real window title bar.
- At widths below 800 points, navigation becomes a horizontal scroll strip.
  Touch controls retain a minimum height of 44 points; desktop actions are 32.
- Empty search states explain the next action. With a library, show a search
  invitation; without one, offer dictionary import. No sample definitions or
  fabricated history appear as if they were user data.
- Definition HTML remains in its existing WebView. Dictionary-provided CSS and
  media keep their original rendering; it is not forcibly recolored in dark mode.

## Visual system

Use system fonts, 13–15 point controls and body text, 24-point section headings,
6–8 point control radii, and 8-point-based spacing. Light mode uses warm neutral
surfaces; dark mode uses separated charcoal surfaces. Blue is reserved for
primary actions and selection. Thin separators replace nested floating cards.
Original sidebar icons are drawn with the native Canvas, without emoji or
platform-dependent icon fonts.

## Language and interaction

English is the default across all three clients, independently of device locale.
Dictionary entries and user-authored notes retain their original language.
Command/Control + 1 through 6 switch destinations. Command/Control + S saves
notes on the Notes page. Return submits the search field. Shortcut labels use
the current platform's conventions. Success feedback clears after 3.5 seconds;
errors remain visible until the next action.

## Verification boundaries

NSH checks cover desktop, iOS, and Android. Headless UI tests cover navigation,
phone-width adaptation, English copy under a Chinese locale, and dark-mode
keyboard navigation. The headless host does not supply the native data directory,
so these tests do not certify persisted notes or dictionary import. Native
visual checks and device testing must be reported separately. Mobile system file
picking remains unavailable; URL and sandbox path imports remain supported.
