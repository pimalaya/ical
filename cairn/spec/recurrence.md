---
cairn: spec
capability: recurrence
status: current
---

# Recurrence

Always available and dependency-free, since gating code that pulls in no crate is churn rather than a feature. `IcalRecurRule::parse` decodes `RRULE` text into typed parts (`FREQ`, `UNTIL`, `COUNT`, `INTERVAL`, every `BY` part, `WKST`, plus the RFC 7529 `RSCALE` and `SKIP`), and `IcalRecurExpand` is the lazy iterator yielding the occurrences a rule and a start denote, per RFC 5545 3.3.10.

The raw rule text stays on `IcalRecur` in the decoded model, which is what byte-faithful round-tripping needs. This layer sits above it and is not the round-trip path, so a part it ignores is dropped rather than carried.

### Requirement: Civil expansion

Expansion SHALL be civil: no time zone, no offset. RFC 5545 defines expansion on the local wall-clock time of `DTSTART`, so no UTC offset is ever needed and none is ever resolved. Turning an occurrence into an instant is the step after it, served by [tz](./tz.md) for one time and by the walk of a series over a window of instants for a whole series (below). The times a set compares are first told on one clock (see below), which is a reading of the calendar, not a change to how it expands.

A zone MAY be supplied to an expansion, and SHALL then be consulted for one purpose only: to drop the instances RFC 5545 3.3.10 forbids counting. It SHALL never change how an occurrence is represented or how the walk steps, both of which stay total arithmetic on civil times.

#### Scenario: A daily rule on a zoned start
- GIVEN `DTSTART;TZID=Europe/Paris:20260301T090000` and `FREQ=DAILY`
- WHEN the rule is expanded
- THEN every occurrence is 09:00:00 civil, across a daylight-saving transition

### Requirement: Liberal rule parsing

An unrecognised rule part SHALL be ignored rather than refused, as RFC 5545 requires. A malformed value inside a part the module does understand SHALL be an error of `IcalRecurRule::parse`; reading a component's set SHALL instead drop that part and keep the rest of the rule, so `COUNT=-1` beside a sound `UNTIL` still bounds by the `UNTIL`, and validation SHALL report the part ([conformance](./conformance.md)). A rule that breaks a constraint of RFC 5545 3.3.10 SHALL still parse: `UNTIL` and `COUNT` together are a validation problem, not a parse error, since strictness belongs on the way out.

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

### Requirement: A walk skips ahead

`IcalRecurExpand::skip_to` and `IcalRecurSetExpand::skip_to` SHALL make the next occurrence yielded the first at or after the given date-time (by identity, for a set), yielding exactly what the full walk yields from there. A rule with no `COUNT` SHALL jump its cursor to the period before the one the date-time falls in, on the `INTERVAL` lattice, so the cost does not grow with the distance from the start; a rule with `COUNT` SHALL walk the earlier occurrences, counted and not yielded, a walk `COUNT` bounds.

#### Scenario: A secondly rule started in 1970
- GIVEN `FREQ=SECONDLY` from `19700101T000000`
- WHEN the walk skips to `20260101T000000`
- THEN the next three occurrences are the first three seconds of 2026, in bounded time

### Requirement: The recurrence set of a component

A component's recurrence set SHALL be the union of its `DTSTART`, every `RRULE` expansion and every `RDATE`, minus every `EXDATE` and every `EXRULE` expansion, yielded in chronological order with duplicates collapsed. An `RDATE` period item SHALL contribute its start.

When the set has rules and every one ends on a day before its `DTSTART`, the `DTSTART` SHALL NOT be an instance: RFC 5545 3.8.5.3 counts it as the first, but such a series is what a client writes when the first instance and every one after it are deleted. The day is compared, so an `UNTIL` written as a date beside a date-time start on the same day cuts nothing. The `RDATE`s still stand.

#### Scenario: A series cut before its start
- GIVEN `DTSTART;VALUE=DATE:20231002` and `RRULE:FREQ=WEEKLY;UNTIL=20231001;INTERVAL=2;BYDAY=MO`
- WHEN its set is expanded
- THEN it yields nothing

#### Scenario: A rule plus an extra date minus an exception
- GIVEN a `VEVENT` carrying one `RRULE`, one `RDATE` and one `EXDATE` falling on a rule occurrence
- WHEN its recurrence set is expanded
- THEN the extra date appears in order and the excepted occurrence does not

### Requirement: A date part on a date-time series names a day

An `EXDATE` item of value type `DATE` beside a `DATE-TIME` `DTSTART` SHALL remove every instance whose identity falls on that day, read on `DTSTART`'s clock (`IcalRecurSet::exdays`). An `UNTIL` of value type `DATE` beside a `DATE-TIME` `DTSTART` SHALL read as the start of that day, excluding the instances later on it. Validation SHALL report both ([conformance](./conformance.md)), RFC 5545 3.3.10 and 3.8.5.1 asking for `DTSTART`'s type. Both readings are the oracles' (libical, recurring-ical-events, ical.js); including the whole `UNTIL` day was declined to stay with them.

#### Scenario: A date exception on a timed series
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000`, `RRULE:FREQ=DAILY;COUNT=4` and `EXDATE;VALUE=DATE:20260707`
- WHEN its set is expanded
- THEN 7 July is not an instance, and 6, 8 and 9 July are

#### Scenario: A date until on a timed series
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000` and `RRULE:FREQ=DAILY;UNTIL=20260708`
- WHEN its set is expanded
- THEN it yields 6 and 7 July at 09:00

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

### Requirement: A set reads every time on its start's clock

A recurrence set SHALL tell every time it holds on one clock, its `DTSTART`'s (`IcalRecurSet::zone`: floating, UTC, or the zone a `TZID` names), since its identities, its exclusions and its bounds are compared with one another.

A time written on another clock SHALL be brought onto that one as the set is read, through the `VTIMEZONE`s that define both clocks, each read as a written `DATE-TIME` (RFC 5545 3.3.5): a UTC `UNTIL`, which RFC 5545 3.3.10 requires beside a zoned `DTSTART`; an `RDATE` or `EXDATE` item in UTC or under another `TZID` (3.8.5.1, 3.8.5.2); and an override's `RECURRENCE-ID` and `DTSTART` (3.8.4.4). `IcalRecurSet::of_uid` SHALL find the zones among the calendar components it is given; `of_uid_in`, `of_component_in` and `with_override_in` SHALL take them, a list the time zone database completes included ([tz](./tz.md)); `IcalRecurOverride::of_component` SHALL read one override onto a given clock without a set.

Without the zones (`of_component`, `with_override`), on a floating clock, or for a zone the calendar does not define, a time SHALL be compared as it is written, which is what the set did before.

#### Scenario: A UTC bound on a zoned series
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000`, `RRULE:FREQ=DAILY;UNTIL=20260708T070000Z` and the Paris `VTIMEZONE`
- WHEN the set of its `UID` is expanded
- THEN 6, 7 and 8 July at 09:00 are its occurrences, 07:00Z being 09:00 in Paris

#### Scenario: An exclusion and an override on other clocks
- GIVEN the same series with an `EXDATE` at `20260707T070000Z` and another under `TZID=America/New_York` at `20260708T030000`, or an override whose `RECURRENCE-ID` is `20260707T070000Z`
- WHEN the set is expanded
- THEN each names the Paris 09:00 instance it means: the exclusions remove theirs, and the override replaces its own rather than standing beside it

#### Scenario: No zones at hand
- GIVEN the UTC-bounded series read with `of_component` alone
- WHEN it is expanded
- THEN the bound is compared as written and 8 July is not an occurrence

### Requirement: The differential corpus is replayed

Expansion SHALL be pinned by a committed corpus of the cases on which python-dateutil and libical answer alike, replayed by the test suite with no Python and no C toolchain present. A second committed corpus SHALL pin each deliberate divergence with this crate's own answer and the reason for it.

#### Scenario: A regression in the expander
- GIVEN a change to the expander that alters an agreed-upon answer
- WHEN the test suite runs
- THEN the consensus replay fails, naming the rule and the start

### Requirement: The timeline corpus is replayed

The walk of a series to instants SHALL be crossed against recurring-ical-events 3.9.0 and ical.js 2.2.1 over the vendored rie, icaljs and libical calendars, and replayed with no Python or JavaScript present. Every series the crate yields SHALL be in one of: agreement.tsv (all three alike), split.tsv (the oracles split; generated with the side this crate takes), divergence.tsv (unlike both; hand-curated with a reason, never unread).

#### Scenario: A regression in the timeline
- GIVEN a change that alters the occurrences of a series both implementations agreed on
- WHEN the test suite runs
- THEN the timeline replay fails, naming the calendar and the `UID`

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

### Requirement: A rule is expanded on its scale, opt-in

`BYMONTH` SHALL parse a month number from 1 to 13 and a leap month (`5L`, RFC 7529 4.2) under every scale, as `IcalRecurMonth`; which months a scale has is read at expansion, and validation SHALL report a 13th or a leap month on the Gregorian scale.

Behind the `rscale` feature, a rule SHALL be expanded on the scale its `RSCALE` names through `icu_calendar`, every `BY` part and `SKIP` read in that calendar, occurrences staying Gregorian civil dates. A leap month a year lacks SHALL move as RFC 7529 4.1 says (`BACKWARD` to the month it follows, `FORWARD` to the next), before the day is resolved. The covered scales are those `IcalRecurExpand::new` lists.

Without the feature, or for a scale it does not cover (`ISLAMIC`, `ISLAMIC-RGSA`), a non-Gregorian `RSCALE` SHALL parse and yield nothing, never Gregorian dates under another scale. The bare core SHALL stay dependency-free.

#### Scenario: A Hebrew anniversary in a leap month (RFC 7529 4.3.3)
- GIVEN `DTSTART;VALUE=DATE:20140208`, `RRULE:RSCALE=HEBREW;FREQ=YEARLY;BYMONTH=5L;BYMONTHDAY=8;SKIP=FORWARD` and the `rscale` feature
- WHEN it is expanded
- THEN it yields 2014-02-08, 2015-02-27, 2016-02-17, 2017-03-06 and 2018-02-23, the 8th of Adar in the years with no Adar I

#### Scenario: Chinese New Year (RFC 7529 4.3.1)
- GIVEN `DTSTART;VALUE=DATE:20130210`, `RRULE:RSCALE=CHINESE;FREQ=YEARLY` and the `rscale` feature
- WHEN it is expanded
- THEN it yields 2013-02-10, 2014-01-31, 2015-02-19, 2016-02-08 and 2017-01-28

#### Scenario: A calendar system the build does not compute
- GIVEN `RSCALE=HEBREW;FREQ=YEARLY` without the `rscale` feature, or `RSCALE=ISLAMIC;FREQ=YEARLY` with it
- WHEN it is expanded
- THEN nothing is yielded

### Requirement: The RSCALE corpus is replayed

A generated `RSCALE` sweep SHALL be frozen where libical with ICU answers alike (tests/corpus/recur/rscale.tsv), each class of disagreement pinned with a reason (rscale-divergence.tsv), and replayed behind the `rscale` feature.

#### Scenario: A change in what a scaled rule denotes
- GIVEN the frozen RSCALE corpora
- WHEN the expander answers differently on any case
- THEN the suite fails, naming the start and the rule

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

### Requirement: A series walks to instants over a window

`IcalRecurTimeline::of_uid` SHALL read the series of one `UID` among a calendar's components, its overrides included, through a given list of zones, and `IcalRecurTimeline::expand` SHALL walk it over a window `[from, until)` of Unix seconds, lazily and in identity order, to occurrences carrying their identity (civil on the series' clock, and as an instant), their start and their end as instants, whether an override placed them, whether they were read on no zone, and whether they are all-day.

Each civil time SHALL be resolved through the zone its own clock names only once the set has placed the occurrence, as a written `DATE-TIME` (RFC 5545 3.3.5); an instance a rule generates in a gap is dropped as recurrence drops it. A floating or date time, or one under a `TZID` none of the zones defines, SHALL be read as if on UTC and flagged floating, so its civil time reads back from its instant exactly.

The end SHALL come from the occurrence's own component: its `DTEND` (a `VTODO`'s `DUE`) on its own clock, else its `DURATION`, else RFC 5545 3.6.1's default, a day for a `VEVENT` on a date and none otherwise. A `DTEND` span SHALL apply to every other instance as the exact seconds it measures (RFC 5545 3.8.5.3), an `RDATE` period's own span to its instance, and an instance a `RANGE=THISANDFUTURE` override shifts SHALL end as that override's own component does, the latest in force winning. A nominal duration SHALL be added in wall-clock days on the start's clock before resolution, an exact one in seconds after. An end before its start SHALL be swapped with it, the identity staying on `DTSTART` so a `RECURRENCE-ID` or an `EXDATE` still names it, and the walk SHALL reach an occurrence whose swapped start falls in the window.

An occurrence SHALL be yielded when it starts before `until` and ends after `from`, or, lasting no time, starts within the window, an override moving an instance in from outside the window included. The walk SHALL skip ahead to the window rather than walk from `DTSTART`, and SHALL bound nothing itself: a consumer bounding what one series yields takes from it. A series with no start, a `DTSTART` or a task's `DUE`, SHALL yield nothing and not fail. The same components, zones and window SHALL always yield the same occurrences.

#### Scenario: A one-day duration across a transition
- GIVEN `DTSTART;TZID=Europe/Paris:20261024T120000`, `DURATION:P1D` and the Paris `VTIMEZONE`
- WHEN the series is walked over that weekend
- THEN the occurrence ends at 12:00 Paris time on 25 October, 25 hours after it starts

#### Scenario: An exact duration across the same transition
- GIVEN the same start and `DURATION:PT24H`
- WHEN the series is walked
- THEN the occurrence ends 24 hours after it starts, at 11:00 Paris time

#### Scenario: An override in another zone
- GIVEN a daily Paris series and an override moving one instance to `TZID=America/New_York`
- WHEN the series is walked
- THEN that occurrence's start and end are the New York instants its own component states, and its identity is still the Paris instance it replaced

#### Scenario: An end in another zone
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000`, `DTEND;TZID=America/New_York:20260706T090000` and a daily rule
- WHEN the series is walked
- THEN every occurrence lasts six hours, from 07:00Z to 13:00Z on its day

#### Scenario: An override moved in from years away
- GIVEN a weekly series from 2020 and an override moving its second instance to 8 July 2026
- WHEN the series is walked over 7 to 9 July 2026
- THEN that override is the one occurrence, unless an `EXDATE` removes the instance it names

#### Scenario: An end written before its start
- GIVEN `DTSTART:20260706T090000Z`, `DTEND:20260706T083000Z` and a daily rule
- WHEN the series is walked
- THEN each occurrence runs from 08:30Z to 09:00Z, its identity still 09:00Z

#### Scenario: A tail made longer
- GIVEN a series lasting two hours and a `RANGE=THISANDFUTURE` override moving one instance three hours earlier and making it last seven
- WHEN the series is walked
- THEN every later instance starts three hours earlier and lasts seven hours

### Requirement: The highest SEQUENCE is the revision

Of the components of one `UID` with no `RECURRENCE-ID`, the one with the highest `SEQUENCE` (RFC 5545 3.8.7.4) SHALL be the series, the last listed of equals; of the overrides naming one instance, the highest `SEQUENCE` SHALL win, the first listed of equals; a missing `SEQUENCE` SHALL count as 0. Both `IcalRecurSet::of_uid_in` and `IcalRecurTimeline::of_uid` SHALL read them so.

An override carrying an `RRULE`, with a lower `SEQUENCE` than the series, and naming an instance the series does not generate SHALL be dropped: it is a former series a reschedule left behind. Every other override naming no generated instance SHALL still stand, as above.

#### Scenario: A former series left behind
- GIVEN a series at `SEQUENCE:2` whose `UNTIL` stops on 20 July, and a component at `SEQUENCE:1` carrying an `RRULE` and a `RECURRENCE-ID` of 29 July
- WHEN the series is walked
- THEN 29 July is not an occurrence

### Requirement: A task recurs from its DUE

A `VTODO` with no `DTSTART` SHALL start its recurrence set, its overrides and its occurrences from its `DUE`, as RFC 8984 4.3.3 recurs a Task with no `start`, each occurrence lasting no time. An event with no `DTSTART` SHALL NOT recur from its `DTEND`.

#### Scenario: A weekly task with a due time
- GIVEN a `VTODO` with `DUE:20260710T170000Z`, `RRULE:FREQ=WEEKLY;COUNT=3` and no `DTSTART`
- WHEN it is walked
- THEN it yields 10, 17 and 24 July at 17:00Z

### Requirement: A component with no UID walks alone

`IcalRecurTimeline::of_component` SHALL read one component as a series with no override, so a component carrying no `UID`, which RFC 5545 requires and a calendar in the wild may lack, still has occurrences.
