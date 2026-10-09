---
cairn: log
change: a-written-time-in-a-gap-still-happens
date: 2026-10-09
---

# A written time in a gap still happens

RFC 5545 gives a local time in a gap two answers by who wrote it: an instance a rule generates there is no instance (3.3.10), and a `DATE-TIME` somebody wrote there is read at the offset before the gap (3.3.5), so `02:30` on the day New York springs forward is `03:30` EDT. `IcalTzOffset::instant` only gave the first, and the Android app resolved every literal time by hand.

`IcalTzOffset::literal_offset` and `IcalTzOffset::literal_instant` give the second: the offset in force, the earlier of a fold's two, the offset before a gap. `instant` is documented as the generated reading and delegates to the literal one off the gap. `IcalTz::local` is the inverse the app guessed at in two steps: the local time a zone shows at an instant, read off the transition in force, which is never ambiguous. The recurrence set reads its foreign-clock times through both (a-recurrence-set-reads-on-one-clock).

Capabilities moved: tz.
