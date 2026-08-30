---
cairn: change
id: a-gap-instance-is-not-counted
status: landed
created: 2026-08-30
---

# A gap instance is not counted

## Why

RFC 5545 section 3.3.10 closes with a rule the crate does not implement:

> Recurrence rules may generate recurrence instances with an invalid date (e.g., February 30) or nonexistent local time (e.g., 1:30 AM on a day where the local time is moved forward by an hour at 1:00 AM). Such recurrence instances MUST be ignored and MUST NOT be counted as part of the recurrence set.

The first half is already right: an invalid date never arises, because expansion generates candidates from the calendar rather than filtering them, so February never yields a 30th and no zone is involved.

The second half is not. A local time the clock jumped over is emitted like any other, and `IcalTzOffset::Gap` is documented as a choice the caller makes. It is not a choice. The RFC settled it, and the documentation says the opposite of the specification.

The counting half is worse than the emitting half. `MUST NOT be counted` means a dropped instance consumes no `COUNT` slot, so a rule bounded by `COUNT=5` whose third instance falls in a gap yields five occurrences and runs one period further. Filtering after expansion cannot produce that: `COUNT` is spent inside `next`, so a post-filter yields four.

## What

The zone enters expansion as a predicate and nothing else. `IcalRecurExpand` gains an optional zone, consulted once per candidate to ask whether that wall clock exists, and a candidate that does not is skipped before `emitted` rises. Occurrences stay `IcalRecurDateTime`, stepping stays total arithmetic, and no value ever crosses a partial function. This is the whole concession, and it is a boolean per candidate rather than a change of representation.

The filter sits on the rule streams alone. The RFC says *recurrence rules generate* such instances; an `RDATE` does not generate, it names, so a date written into an `RDATE` is a deliberate value like a lone `DTSTART` and is not dropped.

`IcalTzOffset` gains `instant`, the crossing named once: `One` subtracts its offset, `Gap` is `None` because the RFC says so, and `Fold` takes the earlier of its two, which the RFC does not mandate and which the variant's own fields still expose. Its documentation says which of the three answers is the specification's and which is a default.

`IcalTz` materialises its transitions when it is read rather than re-expanding every observance on every call, without which a per-candidate predicate would re-derive a zone's whole history once per date.

## Consequence

An expansion given no zone behaves exactly as it does today, which is what every caller before this change gets.

A rule whose every candidate falls in a gap, `FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU;BYHOUR=2;BYMINUTE=30` against the zone whose transition that is, walks to the year cap and yields nothing. The barren-period budget does not catch it, the periods being full of candidates that die at emit time, so the year cap is the bound that matters and the code should say so where it is not obvious.
