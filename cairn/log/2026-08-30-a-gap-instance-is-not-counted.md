---
cairn: log
change: a-gap-instance-is-not-counted
date: 2026-08-30
---

# A gap instance is not counted

RFC 5545 3.3.10 closes with a rule the expander did not implement: a recurrence instance at a nonexistent local time "MUST be ignored and MUST NOT be counted as part of the recurrence set". The first half of that sentence, the invalid date, was already right, because expansion generates candidates from the calendar rather than filtering them, so February never yields a 30th. The second half was not. A local time the clock jumped over came out like any other, and `IcalTzOffset::Gap` documented that as a choice belonging to the caller, which is the opposite of what the specification says.

The counting half was the worse one. `COUNT` is spent inside `next`, so no filter placed after expansion can produce the RFC's answer: a rule bounded by `COUNT=5` whose third instance falls in a gap has to yield five and run one period further, and a post-filter yields four.

The zone therefore enters expansion, and enters it as a predicate and nothing else. `IcalRecurExpand::in_zone` takes an `IcalTz` owned, so the iterator keeps no lifetime and stays the type it was, and one candidate per step is asked whether its wall clock exists. A candidate that does not is skipped before `emitted` rises and before either bound is consulted. Occurrences are still `IcalRecurDateTime`, stepping is still total arithmetic on civil times, and no value crosses a partial function.

The filter sits on the rule streams alone. `IcalRecurSet::expand_in_zone` passes the zone to every `RRULE` and `EXRULE` stream and to no literal: the RFC's clause is about what rules *generate*, and an `RDATE` names a date as deliberately as a lone `DTSTART` does. Both survive a gap.

Asking once per candidate is a different shape from asking once, which `IcalTz::resolve` was built for: it re-expanded every observance on every call, so a per-candidate predicate would have re-derived a zone's whole history once per date. Transitions are materialised now. `IcalTz::transitions` gives them up to the end of a year, taking one onset past that bound from each observance so the list still answers for the year itself, and `IcalTz::resolve` is one lookup over that list rather than a walk of its own. `IcalTzTransitions` holds a zone and grows the list when a query runs past what it covers, doubling the span each time, so walking a rule forward costs one expansion amortised. Both paths answer through the same private `offset`, so there is one resolution and not two.

`IcalTzOffset::instant` names the crossing once, and says in its own documentation which of its three answers the RFC settled: `One` subtracts its offset, `Gap` is `None` because a local time that never happens names no instant, and `Fold` takes the earlier of its two, which the RFC does not mandate and which the variant's fields still expose to a caller wanting the later one.

An expansion given no zone behaves exactly as it did, which is what every caller before this change gets, and a test pins that alongside the new answer. `UNTIL` is pinned too, and floats the other way: it names an instant rather than a tally, so dropping an instance leaves the end of the series where it was and the count one short. A fold is kept, being a time that happens twice rather than none.

A rule whose every candidate falls in a gap, `FREQ=YEARLY;BYMONTH=3;BYDAY=2SU;BYHOUR=2;BYMINUTE=30` against the zone whose transition that is, yields nothing. The barren-period budget does not catch it, every period being full of candidates that die at emit time, so the year cap is the bound that ends the walk, and the module now says so.

Capabilities moved: recurrence, tz.
