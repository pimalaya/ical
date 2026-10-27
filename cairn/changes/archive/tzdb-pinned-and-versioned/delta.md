---
cairn: delta
change: tzdb-pinned-and-versioned
---

## ADDED Requirements

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

## MODIFIED Requirements

## REMOVED Requirements
