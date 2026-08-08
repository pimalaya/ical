---
cairn: delta
change: recur-skip
---

## ADDED Requirements

### Requirement: SKIP resolves a day that does not exist

A rule carrying `SKIP=BACKWARD` or `SKIP=FORWARD` SHALL resolve a day of the month that the month does not have, rather than dropping the occurrence (RFC 7529 4.1). `BACKWARD` moves it to the last day the month does have; `FORWARD` moves it to the first day of the next month. `SKIP=OMIT`, the default, drops it, which is RFC 5545's own behaviour.

Only a day the rule intends is resolved: each positive `BYMONTHDAY`, or the day the start supplies when no part selects one. A negative `BYMONTHDAY` counts back from the end of a month and always names a day that exists.

A resolved day is not put back through the day-selecting parts, since RFC 7529 resolves it after `BYMONTHDAY` has chosen it and the whole point is to land on a date the rule did not choose. Occurrences stay ordered and are emitted once, so a day two `BYMONTHDAY` values both resolve onto appears once.

`SKIP` SHALL be reported by validation when no `RSCALE` accompanies it (RFC 7529 4), and SHALL still be parsed and expanded, since a rule that says too much is still a rule.

#### Scenario: The leap day of RFC 7529 4.3.4
- GIVEN `DTSTART:20120229` and `RRULE:RSCALE=GREGORIAN;FREQ=YEARLY;SKIP=FORWARD`
- WHEN it is expanded
- THEN the occurrences are 2012-02-29, 2013-03-01, 2014-03-01, 2015-03-01, 2016-02-29

#### Scenario: The thirty-first of every month
- GIVEN `DTSTART:20260131` and `RRULE:RSCALE=GREGORIAN;FREQ=MONTHLY;SKIP=BACKWARD`
- WHEN it is expanded
- THEN February yields the 28th and April the 30th

#### Scenario: SKIP with no RSCALE
- GIVEN `FREQ=YEARLY;SKIP=FORWARD`
- WHEN the rule is validated
- THEN the missing `RSCALE` is reported, and expansion still resolves the day

### Requirement: Only the Gregorian scale is expanded

An `RSCALE` naming any calendar system other than `GREGORIAN` SHALL be parsed, carried and converted, and SHALL yield no occurrences. Expanding a Hebrew or Chinese rule needs the CLDR calendar arithmetic RFC 7529 5 points implementers at, which this crate does not carry; yielding Gregorian dates under another scale would be a wrong answer rather than a missing one.

#### Scenario: A calendar system the crate does not know
- GIVEN `RSCALE=HEBREW;FREQ=YEARLY`
- WHEN it is expanded
- THEN nothing is yielded

### Requirement: The SKIP corpus is replayed

The `SKIP` answers SHALL be frozen as a corpus and replayed by the test suite. One oracle rather than the two the main corpus crosses, because only one exists: python-dateutil does not parse `RSCALE`, and libical resolves `SKIP` for the Gregorian scale without an ICU build.

#### Scenario: A change in what a rule denotes
- GIVEN the frozen corpus at tests/corpus/recur/skip.tsv
- WHEN the expander answers differently from libical on any case
- THEN the suite fails

