---
cairn: spec
capability: tz
status: current
---

# Time zones

Turning a civil date-time into a UTC offset, using the `VTIMEZONE` the calendar carries, and behind the `tzdb` feature the bundled database for a zone it omits. Expansion stays civil ([recurrence](./recurrence.md)); this is the step after it. A `VTIMEZONE` is read into its `STANDARD` and `DAYLIGHT` observances, each with the offset before it, the offset after it, and the recurrence set saying when it takes effect. Built on the rule expander, and available with it.

### Requirement: Offset resolution from the calendar's own rules

A civil date and time plus a `VTIMEZONE` SHALL resolve to a UTC offset, using only the observances that travel inside the calendar. No time-zone database and no new dependency SHALL be required.

Every `DTSTART` inside an observance is local to the offset *before* the transition (RFC 5545 3.6.5). A local time before the first transition SHALL take the offset that transition states came before it. A zone with no observance at all SHALL resolve to UTC, since it states no offset to apply.

#### Scenario: An unambiguous local time
- GIVEN a `VTIMEZONE` with a `STANDARD` and a `DAYLIGHT` observance
- WHEN a civil time in mid-July is resolved
- THEN the daylight observance's `TZOFFSETTO` is returned

#### Scenario: A zone that never shifts
- GIVEN a `VTIMEZONE` whose single observance moves the clock by nothing
- WHEN any civil time is resolved
- THEN that observance's offset is returned

### Requirement: The gap and the fold are reported

A local time a spring-forward skips SHALL be reported as skipped, and a local time a fall-back gives twice SHALL be reported as ambiguous with both candidate offsets. Neither SHALL be silently resolved to one answer.

#### Scenario: A skipped local time
- GIVEN a transition that jumps 02:00 to 03:00
- WHEN 02:30 is resolved
- THEN the result reports a gap, naming the offsets either side

#### Scenario: A repeated local time
- GIVEN a transition that repeats 02:00 to 03:00
- WHEN 02:30 is resolved
- THEN the result reports a fold, carrying both the earlier and the later offset

### Requirement: The crossing to an instant follows the RFC's two answers

RFC 5545 answers a local time in a gap twice, by who wrote it, and the crossing SHALL follow both.

`IcalTzOffset::instant` SHALL give the instant a local time a rule generated names, as seconds since the Unix epoch. It SHALL return nothing in a gap, which is RFC 5545 3.3.10's answer for an instance rather than a refusal to answer, and the earlier of the two in a fold, which is a default the RFC does not mandate and which the variant's fields still expose.

`IcalTzOffset::literal_offset` and `IcalTzOffset::literal_instant` SHALL read a written `DATE-TIME` (a `DTSTART`, an override's start, an `EXDATE`, a `RECURRENCE-ID`) by RFC 5545 3.3.5: the offset in force, the earlier of a fold's two, and in a gap the offset before it, so a written time always names an instant.

`IcalTz::local` SHALL give the local time a zone's clock shows at an instant, the inverse of a resolution, which is never ambiguous.

#### Scenario: The three answers
- GIVEN a resolution that is one offset, a gap, and a fold
- WHEN each is asked for the instant its local time names
- THEN the offset is subtracted, the gap names none, and the fold takes its earlier offset

#### Scenario: The examples of RFC 5545 3.3.5
- GIVEN America/New_York, and `20070311T023000` and `20071104T013000` written in it
- WHEN each is read as a written time
- THEN the first names 03:30 EDT, at the offset before the gap, and the second 01:30 EDT, the first of the two

### Requirement: A VTIMEZONE can be synthesized from the database, opt-in

Behind the `tzdb` feature, and only there, `tzdb::vtimezone` SHALL build the VTIMEZONE an IANA name stands for from the bundled time zone database, describing the era an anchor instant falls in, and `tzdb::is_known` SHALL tell whether a name can be rebuilt so. A calendar synthesized from a representation that names a zone and stops there owes that component (RFC 5545 3.2.19). The database SHALL stay out of every other feature: offset resolution keeps needing no database and no dependency.

#### Scenario: A zone settled for decades
- GIVEN `Asia/Hong_Kong` and an anchor in 2026
- WHEN its VTIMEZONE is synthesized
- THEN it carries a single standard observance and no summer time

### Requirement: The bundled database names its version

Behind the `tzdb` feature, `tzdb::version` SHALL return the IANA release of the bundled database, so a consumer that materialises instants can record the rules it used and recompute when they change. The database SHALL be the bundled one alone: no system zone file is read, so one crate version and one lock file resolve alike on every machine.

#### Scenario: A stamp a consumer can compare
- GIVEN two builds of the crate bundling the same database
- WHEN `tzdb::version` is read in each
- THEN both answer the same release name, such as `2026c`

### Requirement: A zone the calendar omits resolves from the database, opt-in

Behind the `tzdb` feature, `tzdb::zone` SHALL build the `IcalTz` an IANA name stands for from every transition the database states up to the end of a given year, exact over the whole history to that year, and usable wherever a zone read from a `VTIMEZONE` is. `tzdb::zones` SHALL list a calendar's own zones (`IcalTz::of_components`) followed by a database zone for every `TZID` its components name and no `VTIMEZONE` defines, so a reader taking the first zone of a name resolves through the calendar's own `VTIMEZONE`, else the database, else as floating. A `VTIMEZONE` the calendar carries SHALL always win over the database. Without the feature, or for a name the database does not know, the time SHALL read as it does without a database.

#### Scenario: An event naming a zone it does not carry
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000` and no `VTIMEZONE` in the calendar
- WHEN its occurrences are walked through `tzdb::zones`
- THEN the occurrence starts at 07:00Z, as it would with the Paris `VTIMEZONE`

#### Scenario: The sender's own definition
- GIVEN a calendar whose `VTIMEZONE` for `Europe/Paris` states +0300 all year
- WHEN its occurrences are walked through `tzdb::zones`
- THEN 09:00 in that zone is 06:00Z, the database notwithstanding

#### Scenario: A zone whose rules changed
- GIVEN `tzdb::zone("America/New_York", 2026)`
- WHEN noon on 20 March 2006 and on 20 March 2007 are resolved
- THEN the first is standard time and the second daylight time, the rule having moved in 2007
