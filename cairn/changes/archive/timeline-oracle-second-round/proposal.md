---
cairn: change
id: timeline-oracle-second-round
status: landed
created: 2026-10-29
---

# The timeline oracle, second round

## Why

Once [what-the-timeline-oracle-found](../what-the-timeline-oracle-found/proposal.md) let components with no `UID` be walked, the oracle run showed three more differences in them, and one left over. Settled with the maintainer on 2026-10-29.

## What

1. **A Windows zone name reads as its IANA zone.** `tzdb::zone` maps a name such as `Pacific Standard Time` through the CLDR windowsZones table (release-49, territory 001, 142 names, src/tzdb/windows.rs) when the database does not know it, the zone keeping the name it was asked by. `is_known` and `vtimezone` stay IANA only, since they decide whether the JSCalendar projection may drop a `VTIMEZONE`. (recurring-ical-events issue 107.)
2. **A series cut before its start has no instance.** When every rule of a set ends on a day before its `DTSTART`, the `DTSTART` is no instance either: RFC 5545 3.8.5.3 counts it as the first, but this is what a client writes when the first instance and every one after it are deleted. Days are compared, so a date `UNTIL` on a date-time start's own day cuts nothing. (Issue 117.)
3. **A malformed rule part is dropped, not the rule.** Reading a set, a rule `IcalRecurRule::parse` refuses is read again part by part, keeping every part that parses, so `COUNT=-1` beside a sound `UNTIL` bounds by the `UNTIL`. (Issue 128.) Validation passed such a rule silently, and now reports it as `IcalValidateError::RuleSyntax`.

## Not changed

A `VTIMEZONE` that ends early (fablab_cottbus: dated onsets up to 2019-03-31, no `TZUNTIL`) keeps its last offset past its last onset: a zone with a single observance holds its offset forever, and nothing tells the two apart without `TZUNTIL`. Recorded as a divergence.
