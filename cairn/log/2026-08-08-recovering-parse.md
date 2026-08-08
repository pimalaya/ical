---
cairn: log
change: recovering-parse
landed: 2026-08-08
---

# A recovering parse mode

Twelve of the 190 vendored fixtures were being thrown away whole over one bad line. `IcalCst::parse_recovering` now reads them: it keeps a physical line it cannot structure as an `IcalItem::Opaque`, verbatim with its ending, and carries on. A component still open at end of input is closed with no `END` rather than failing the calendar. The strict entry points are untouched and stay the default.

What comes back is an `IcalRecovery`: every top-level calendar it could read, in source order, and every problem it worked around, in the order it met them. A run of lines outside any `BEGIN` becomes a bare, envelope-less calendar of its own, so garbage between calendars is held rather than dropped. `to_bytes` concatenates the lot, and that is the guarantee the tests pin: all 190 fixtures, the twelve broken ones included, serialize back to their input byte for byte, and reparsing that output recovers the same calendars with the same problem count.

The recovery step is deliberately one *physical* line, not one logical one. A folded line whose joined form has no colon becomes an opaque first line followed by its continuations, each of which then fails on its own and becomes opaque in turn. That is uglier to describe than joining them first, and it is the version that reaches a fixpoint: the output reparses to exactly itself.

`IcalParseError::MissingEnd` now carries the name of the component that was left open rather than the whole input. It was carrying a copy of the entire calendar, which for the 1 MB libical fixture meant a 1 MB error string. That is a diagnostic nobody can read and an allocation nobody wanted.

Counts, per corpus: libical 30 clean and 10 repaired, ical4j 102 and 2, ical.js 46 and 0, the RFC fixtures 6 and 0.

Capabilities moved: `parsing` (ADDED: recovering parse).
