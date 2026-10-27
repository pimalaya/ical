---
cairn: log
change: occurrences-as-instants
date: 2026-10-27
---

# Occurrences as instants

Expansion stopped at civil identities and starts, so every caller wanting a window of instants (the Android bridge's rust/src/calendar/expand.rs, pimdir's search index, an agenda) resolved zones and placed ends itself, each meeting `P1D` across a transition, an override moved to another zone and an end in another zone on its own.

A new module, `recur::instant`, always available and dependency-free:

- `IcalRecurTimeline::of_uid(components, uid, zones)` reads one `UID`'s series and overrides through a zone list (`IcalTz::of_components`, or `tzdb::zones` with the database), owning what it needs.
- `IcalRecurTimeline::expand(from, until)` walks it over `[from, until)` in Unix seconds to `IcalRecurInstant { id, id_instant, start, end, over, floating, date }`, lazily, in identity order, skipping to the window with `skip_to`; an override moved in from outside the window is placed apart and yielded unless an `EXDATE` or `EXRULE` removes its instance.
- The end is the component's own: `DTEND` (`DUE` for a task) on its own clock, else `DURATION`, else RFC 5545 3.6.1's default; a `DTEND` span applies to the other instances in exact seconds (3.8.5.3), an `RDATE` period's span to its instance. `IcalDuration::nominal` splits a duration into wall-clock days and exact seconds, so `P1D` across the Paris fall-back lasts 25 hours and `PT24H` 24; `IcalDuration::seconds` now reads through it.
- `IcalRecurSet::of_uid_in` reads a set through a given zone list, which `of_uid` now calls with the calendar's own.

Decisions, recorded in the proposal: a floating or date time, or one under an undefined `TZID`, reads as if on UTC and is flagged, so its civil time reads back exactly and a viewer's clock is applied by the consumer; the walk bounds nothing itself, a consumer `take`s; the window is an overlap; a task with a `DUE` and no `DTSTART` is one point; an override with no series is still an instance, as recurrence already stated, which differs from pimdir SEARCH §7's "a resource holding overrides and no master yields no occurrence".

Nineteen tests in tests/recur_instant.rs, five behind `tzdb`; each was checked to fail with the code it covers mutated. The differential check against the Android bridge is left to the bridge's move onto this.

Capability moved: **recurrence** (a series walks to instants over a window; civil expansion and the one-clock reading reworded for it).
