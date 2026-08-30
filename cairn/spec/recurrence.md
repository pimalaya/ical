---
cairn: spec
capability: recurrence
status: current
---

# Recurrence

Always available and dependency-free, since gating code that pulls in no crate is churn rather than a feature. `IcalRecurRule::parse` decodes `RRULE` text into typed parts (`FREQ`, `UNTIL`, `COUNT`, `INTERVAL`, every `BY` part, `WKST`, plus the RFC 7529 `RSCALE` and `SKIP`), and `IcalRecurExpand` is the lazy iterator yielding the occurrences a rule and a start denote, per RFC 5545 3.3.10.

The raw rule text stays on `IcalRecur` in the decoded model, which is what byte-faithful round-tripping needs. This layer sits above it and is not the round-trip path, so a part it ignores is dropped rather than carried.

### Requirement: Civil expansion

Expansion SHALL be civil: no time zone, no offset. RFC 5545 defines expansion on the local wall-clock time of `DTSTART`, so no UTC offset is ever needed and none is ever resolved. Turning an occurrence into an instant is the caller's step, served by [tz](./tz.md).

A zone MAY be supplied to an expansion, and SHALL then be consulted for one purpose only: to drop the instances RFC 5545 3.3.10 forbids counting. It SHALL never change how an occurrence is represented or how the walk steps, both of which stay total arithmetic on civil times.

#### Scenario: A daily rule on a zoned start
- GIVEN `DTSTART;TZID=Europe/Paris:20260301T090000` and `FREQ=DAILY`
- WHEN the rule is expanded
- THEN every occurrence is 09:00:00 civil, across a daylight-saving transition

### Requirement: Liberal rule parsing

An unrecognised rule part SHALL be ignored rather than refused, as RFC 5545 requires. A malformed value inside a part the module does understand SHALL be an error. A rule that breaks a constraint of RFC 5545 3.3.10 SHALL still parse: `UNTIL` and `COUNT` together are a validation problem, not a parse error, since strictness belongs on the way out.

#### Scenario: A rule bounded twice
- GIVEN `FREQ=DAILY;COUNT=10;UNTIL=20260101T000000Z`
- WHEN it is parsed
- THEN it parses, and validation reports the two bounds

### Requirement: BYDAY ordinal scope

A `BYDAY` ordinal SHALL be scoped by frequency alone: the month at `MONTHLY`, the year at `YEARLY`, narrowed to the month when `BYMONTH` picks the months of a yearly period. The presence of another `BY` part SHALL NOT void it, and outside `MONTHLY` and `YEARLY` an ordinal SHALL be ignored.

#### Scenario: An ordinal beside BYMONTHDAY
- GIVEN `FREQ=MONTHLY;BYMONTHDAY=15;BYDAY=2MO`
- WHEN the rule is expanded
- THEN an occurrence is yielded only when the 15th is also the second Monday

### Requirement: Bounded work per occurrence

Every call to `next` SHALL do bounded work, whatever the shape of the rule, and no satisfiable rule SHALL be cut short. Expansion SHALL be bounded by the year 9999 cap and by one budget of barren periods per occurrence, shared by the fill and seek paths, so a rule naming a date no calendar has, and one whose `BYSETPOS` can never be filled, both end rather than hang.

#### Scenario: A rule that never yields
- GIVEN `FREQ=SECONDLY;BYSETPOS=2`
- WHEN the iterator is asked for its first occurrence
- THEN it returns none in bounded time rather than walking the calendar a second at a time

#### Scenario: A sparse rule that does yield
- GIVEN `FREQ=MONTHLY;INTERVAL=7;BYMONTH=6;BYDAY=SU;BYMONTHDAY=1`
- WHEN the iterator is driven past the year 2183
- THEN it keeps yielding, since the rule is satisfiable

### Requirement: The recurrence set of a component

A component's recurrence set SHALL be the union of its `DTSTART`, every `RRULE` expansion and every `RDATE`, minus every `EXDATE` and every `EXRULE` expansion, yielded in chronological order with duplicates collapsed. An `RDATE` period item SHALL contribute its start.

#### Scenario: A rule plus an extra date minus an exception
- GIVEN a `VEVENT` carrying one `RRULE`, one `RDATE` and one `EXDATE` falling on a rule occurrence
- WHEN its recurrence set is expanded
- THEN the extra date appears in order and the excepted occurrence does not

### Requirement: Set expansion stays lazy

The recurrence set SHALL be produced by a lazy merge of sorted streams, materialising no occurrence list, so an unbounded rule can be taken from without running to its end.

#### Scenario: An unbounded rule
- GIVEN a `VEVENT` whose `RRULE` has neither `UNTIL` nor `COUNT`
- WHEN the first ten occurrences are taken
- THEN only bounded work is done

### Requirement: Overrides replace instances

Every occurrence SHALL carry both the identity the rules place it at, which is what a `RECURRENCE-ID` names and an `EXDATE` removes, and the start it actually happens at. A `RECURRENCE-ID` override SHALL replace the instance it names, and an override naming an instance no rule generates SHALL still be an instance. With `RANGE=THISANDFUTURE` an override SHALL also shift every later instance by the offset it applies to its own start.

Occurrences come out in the chronological order of their identity, which is what keeps the walk lazy. An override that moves an instance is emitted in the place of the instance it replaces, so its start may fall out of order.

#### Scenario: A single moved instance
- GIVEN an override whose `RECURRENCE-ID` names the third occurrence and whose `DTSTART` is later
- WHEN the set is expanded
- THEN the third occurrence is the overridden one, keeping the identity the rule gave it, and the others are unchanged

#### Scenario: A moved tail
- GIVEN the same override carrying `RANGE=THISANDFUTURE`
- WHEN the set is expanded
- THEN the third and every later occurrence are shifted by the same offset

### Requirement: The differential corpus is replayed

Expansion SHALL be pinned by a committed corpus of the cases on which python-dateutil and libical answer alike, replayed by the test suite with no Python and no C toolchain present. A second committed corpus SHALL pin each deliberate divergence with this crate's own answer and the reason for it.

#### Scenario: A regression in the expander
- GIVEN a change to the expander that alters an agreed-upon answer
- WHEN the test suite runs
- THEN the consensus replay fails, naming the rule and the start

### Requirement: BYWEEKNO uses the ISO week-year

A `BYWEEKNO` week that straddles a year boundary SHALL be assigned by ISO week-year, so the days either side of January 1 belong to the year their week belongs to.

This is a deliberate divergence from libical, which numbers weeks its own way and disagrees on 70 cases of the corpus. python-dateutil agrees with this reading on every comparable case there. The ISO reading is what the crate's own week numbering implements and what the RFC 5545 worked example implies.

### Requirement: BYSETPOS applies to the whole period

`BYSETPOS` SHALL be applied to the whole candidate set of a period, and `DTSTART` SHALL then drop what precedes it, rather than the period being truncated at `DTSTART` first.

Both oracles read it the same way. It is recorded because RFC 5545 leaves the reading to the implementer and the two orders give different answers for the first period.

### Requirement: BYSETPOS is honoured at DAILY, WEEKLY and HOURLY

`BYSETPOS` SHALL be honoured at every frequency RFC 5545 permits it at, `DAILY`, `WEEKLY` and `HOURLY` included. A position no candidate set holds SHALL yield nothing, rather than the part being dropped.

This is a deliberate divergence from libical, which ignores `BYSETPOS` there and is the single largest source of disagreement in the corpus, 239 cases. python-dateutil agrees with this crate, and so do the CalDAV servers and clients in the field.

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

### Requirement: A part its frequency forbids is ignored

A `BY` part at a frequency RFC 5545 3.3.10 forbids it at SHALL be ignored by expansion, exactly as if it were absent: not applied as a limit, not applied as an expansion, and not refused. This covers `BYWEEKNO` outside `YEARLY`, `BYYEARDAY` at `DAILY`, `WEEKLY` and `MONTHLY`, `BYMONTHDAY` at `WEEKLY`, and a `BYDAY` ordinal outside `MONTHLY` and `YEARLY`.

Ignoring it whole also means it cannot stop `DTSTART` from supplying the field the frequency reads off it: a monthly rule keeps its day of the month whatever forbidden parts sit beside it.

This is a deliberate divergence. python-dateutil applies such a part as a limit, libical refuses the rule at parse time. Ignoring is what "liberal in what it accepts" implies, and it is the half of the split that [conformance](./conformance.md) completes: validation reports the part, expansion ignores it.

#### Scenario: BYWEEKNO at a monthly frequency
- GIVEN `FREQ=MONTHLY;BYWEEKNO=3` from a start on the 15th
- WHEN the rule is expanded
- THEN it yields the 15th of each month, as `FREQ=MONTHLY` alone would

### Requirement: A nonexistent local time is dropped and not counted

A rule-generated instance whose local time does not exist in the zone the component references SHALL be omitted from the recurrence set, and SHALL NOT consume a `COUNT` slot (RFC 5545 3.3.10). A rule bounded by `COUNT` therefore yields as many occurrences as it names, running further in time to do so.

The rule SHALL apply to instances a rule generated. A date named by an `RDATE` SHALL be kept, the specification's clause being about what rules generate rather than about every date-time in a gap.

An expansion given no zone SHALL behave as it did before, since a validity it cannot check is one it does not apply.

#### Scenario: A COUNT slot a gap does not consume
- GIVEN a daily rule at a local time the clock jumps over, bounded by `COUNT=5`
- WHEN it is expanded in the zone that jumps
- THEN five occurrences come out, and the series runs one period past where it would have ended

#### Scenario: A date an RDATE names in a gap
- GIVEN an `RDATE` naming a local time the clock jumps over
- WHEN the set is expanded in that zone
- THEN the date is kept, having been named rather than generated
