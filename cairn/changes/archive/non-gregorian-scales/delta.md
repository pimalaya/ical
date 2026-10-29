---
cairn: delta
change: non-gregorian-scales
---

## MODIFIED Requirements

### Requirement: Only the Gregorian scale is expanded

In [recurrence](../../spec/recurrence.md). Without the `rscale` feature, an `RSCALE` other than `GREGORIAN` SHALL still be parsed, carried and converted, and SHALL yield no occurrences. With it, a rule SHALL be expanded on the scale it names, years, months, days, leap months (`BYMONTH=5L`) and `SKIP` resolved in that calendar, and each date SHALL reach the rest of the crate as a Gregorian civil date. A scale the feature does not cover SHALL keep yielding nothing.

#### Scenario: A Hebrew yearly rule
- GIVEN `RRULE:RSCALE=HEBREW;FREQ=YEARLY;BYMONTH=5L;BYMONTHDAY=8;SKIP=FORWARD` and the `rscale` feature
- WHEN it is expanded
- THEN in a Hebrew year with no Adar I it yields the day `SKIP=FORWARD` names, as libical with ICU does
