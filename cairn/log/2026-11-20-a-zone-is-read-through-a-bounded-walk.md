---
cairn: log
change: a-zone-is-read-through-a-bounded-walk
landed: 2026-11-20
---

# A zone is read through a bounded walk

Found by the pimdir freeze plan's hostile vectors (G8, high, denial of service), with pimdir's vectors/fixtures/event-zoned-hostile.ics: a `DAYLIGHT` observance with `RRULE:FREQ=SECONDLY` from 1970. `IcalTz::transitions` walked every onset of every observance up to the year after the one asked, about 1.8e9 for June 2026, so any resolution of a time in that zone exhausted memory (22 GB RSS, then a 10 GiB allocation abort in io-pimdir, through `PimdirIndex::refresh`, `IcalRecurTimeline::of_uid` and `IcalTzTransitions::resolve`). `COUNT=4000000000` did the same, and under a name the database knows `tzdb::zones` walked it again while completing the zone (`before`, `in_force`). The 2026-10-11 skip (hostile-input-bounds) bounded walks of an event's own rule, not of a zone's; io-pimdir's 2026-11-17 hostile-interval fix bounded its own summary walk, not this one.

Resolving against each observance without materialising its history (the last onset at or before a time found by bisecting `skip_to`) was weighed and set aside: exact for any rule without `COUNT`, but a `COUNT` rule cannot skip, each resolution would cost dozens of fresh expansions per observance instead of a lookup, and the materialised list `IcalTzTransitions` amortises would need windows with their own bound anyway. A count bound keeps every real zone's answer and its cost exactly as before.

## What landed

- src/tz.rs: `IcalTzObservance::ONSETS_MAX` (16384) and `IcalTzObservance::walk`, an observance's onsets in order, the first `ONSETS_MAX` of them; `IcalTz::TRANSITIONS_MAX` (65536), the budget one `transitions` call walks over all observances, in source order. `resolve`, `local` and `IcalTzTransitions` read through it unchanged.
- src/tzdb.rs: `before` and `in_force` walk through `walk` within the same budget; `truncated` reads `transitions`.
- Real zones are untouched: one shifting twice a year from 1601 to 9999 states 8399 onsets per observance, 16798 in all, and a database zone built through 9999 at most 16163 (Europe/London, the largest of ten measured with `tzdb::zone(name, 9999)`), 8078 per observance; a known name completed from the database holds both, still under the budget.
- What a hostile zone answers past the bounds is deterministic: the pimdir fixture resolves noon on 1 June 2026 to +0000, its daylight observance read through 04:33:03 on 1 January 1970 and its standard one in force from 1980.

## Tests

tests/tz.rs: `an_observance_taking_effect_every_second_resolves_on_a_bounded_walk` (the fixture's zone, plain and with `COUNT=4000000000`, through `resolve`, `local`, `IcalTzTransitions` and `IcalRecurTimeline`) and `a_hostile_observance_under_a_database_name_resolves_on_a_bounded_walk` (the zone named Europe/Paris, with `TZUNTIL` and with `COUNT`, through `tzdb::zones`) both aborted on 8ea6922 under `ulimit -v 4000000` ("memory allocation of 5368709120 bytes failed", then 6442450944 after 60 s); they pass in 0.15 s in a debug build now. `a_zone_ruled_from_1601_still_shifts_in_9999` guards the bound from below: it fails with `ONSETS_MAX` at 4096.

## Left

Each of the at most 65536 onsets a reading walks can still cost the expander's own bound of a million barren periods (recurrence, "Unsatisfiable rules end"), a CPU bound that predates this change. `IcalTz::resolve` and `IcalTz::local` still repeat their walk on every call, as before; `IcalTzTransitions` is the shape for many calls.

## Capabilities moved

tz ("A zone is read through a bounded walk", added).
