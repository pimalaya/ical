---
cairn: change
id: occurrences-as-instants
status: landed
created: 2026-10-18
---

# Occurrences as instants, in three layers

## Why

The crate expands civilly and stops there: `IcalRecurSetExpand` yields an `IcalRecurOccurrence` holding the instance identity and its civil start, and turning that into an instant, and giving the occurrence an end, is every caller's step. Every caller that needs a window of instants writes it again: the Android bridge (rust/src/calendar/expand.rs), a search index materialising occurrences, an agenda. Each gets the hard parts on its own: a `DURATION` of `P1D` across a daylight-saving change, an override moving one instance to another zone, an end in a zone other than the start's.

An external engine splits expansion into three layers, and two of them are already here:

1. **Civil generation**, pure arithmetic with no zone (recurrence: civil expansion). Done.
2. **Zone resolution to an instant**, deferred until the occurrence is placed. Partly done: `IcalTzOffset::instant` resolves one civil time, a gap answering none (tz). Not wired to the set.
3. **The fold of overrides and exclusions** on one clock (recurrence: overrides replace instances; a set reads every time on its start's clock). Done.

What is missing is the third step's output carried through the second: an occurrence with a start and an end as instants.

## What

A new module, `recur::instant`, always available and dependency-free:

- `IcalRecurTimeline::of_uid(components, uid, zones)`: the series of one `UID` among a calendar's components, overrides included, every time resolved through `zones` (the calendar's own, `IcalTz::of_components`, or those and the database's, `tzdb::zones`).
- `IcalRecurTimeline::expand(from, until)`: the lazy walk of the occurrences overlapping the window `[from, until)` of Unix seconds, each an `IcalRecurInstant`: its identity (civil on the series' clock, and as an instant), its start and its end as Unix seconds, whether an override placed it, whether it was read on no zone, whether it is all-day.
- **The end** from the instance's own component: `DTEND` (`DUE` for a `VTODO`) on its own clock, else `DURATION`, else RFC 5545 3.6.1's default (one day for a `VEVENT` on a date, none otherwise); an `RDATE` period's own end.
- **Spans by calendar math**: a nominal duration (`P1D`, `P1W`) adds wall-clock days on the start's clock and resolves after, so a day across a transition is 23 or 25 hours; an exact one (`PT24H`) adds seconds (RFC 5545 3.3.6). Never `+86400` for a day.
- A start in a gap is dropped as recurrence already drops it; a written time in a gap still happens (tz); a fold takes the earlier instant.
- **Determinism**: one component, one set of zones, one window give identical occurrences, the window skipping ahead with `skip_to` rather than walking from `DTSTART`.

## Decisions

- **Floating times read on UTC, flagged.** A floating or date start is resolved as if UTC, so its instant is its civil second count, and the occurrence says `floating`. A consumer showing it on a viewer's clock reads it back exactly (`IcalRecurDateTime::from_seconds(start)`) and resolves it there. No viewer zone is taken: an index materialises once for every viewer, and a zone passed in would bake one viewer's clock into every row.
- **An all-day occurrence** is the same: instants at UTC midnight, flagged `date` and `floating`, which is a civil date range losslessly encoded.
- **No bound inside the walk.** It is a lazy iterator, so a consumer bounding what one item writes takes `take(n)` and asks one more to know it was cut. Every step does the bounded work recurrence already guarantees.
- **The window is an overlap**: an occurrence is yielded when it starts before `until` and ends after `from`, or, zero-length, starts in `[from, until)`. Occurrences come in identity order, as the set yields them, an override moved in from outside the window included.
- **A `DTEND` span is exact for the other instances** (RFC 5545 3.8.5.3: "the same exact duration will apply to all the members of the generated recurrence set"), a `DURATION` nominal or exact as written. An instance shifted by a `RANGE=THISANDFUTURE` override keeps the series' span. An end before its start is read as the start.
- **A task with a `DUE` and no `DTSTART`** is one zero-length occurrence at its due time, so a task is placed by its `DUE`; with an `RRULE` too it is a series no expander accepts, and yields nothing, as does any `RRULE` with no `DTSTART`.
- **Overrides with no series** are instances, as recurrence already states ("an override naming an instance no rule generates SHALL still be an instance"): a calendar user invited to one instance of a meeting receives that override alone. A consumer that wants them dropped checks for the series itself.
