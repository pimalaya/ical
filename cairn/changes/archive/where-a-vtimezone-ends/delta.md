---
cairn: delta
change: where-a-vtimezone-ends
---

## MODIFIED Requirements

### Requirement: A zone the calendar omits resolves from the database, opt-in

In [tz](../../spec/tz.md). The sentence "`TZUNTIL`, the other end of that span, is not read." becomes: the span SHALL end at the `VTIMEZONE`'s `TZUNTIL` when it states one, and, when no observance recurs without end, after its latest onset; past that end, a name the database knows SHALL read through the database's own transitions. A definition whose zone really stopped changing reads as before, the database stating no later onset either.

#### Scenario: A VTIMEZONE truncated to recent years, read later
- GIVEN a `VTIMEZONE` for `Europe/Berlin` whose observances are dated only and end with the onset of 31 March 2019, and an event at 14:00 Berlin time on 4 December 2023
- WHEN it is walked through `tzdb::zones`
- THEN it starts at 13:00Z, winter time as the database states it, not at the +0200 the last observance came to

#### Scenario: A definition that states its end
- GIVEN a `VTIMEZONE` for `Europe/Berlin` with `TZUNTIL:20200101T000000Z` and a yearly `RRULE` per observance
- WHEN a time in 2024 is walked through `tzdb::zones`
- THEN it reads through the database
