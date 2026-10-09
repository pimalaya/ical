---
cairn: spec
capability: conformance
status: current
---

# Conformance

Strictness on the way out, as two runtime steps over one source of truth. Each property carries an `IcalPropSpec` on the marker it defines in `prop`, and each component an `IcalComponentSpec` on the marker it defines in `component`. A single vtable dispatch bridges the open kinds back to those static specs, so the decoder, the validator and the builder all read the same table.

A contract is what the RFC allows, so it is model rather than syntax: neither the markers, the vtable, the validator nor the builder requires the `parser` feature, and only the read-and-edit lens on a property marker sits under `tree`.

### Requirement: Validation is a runtime predicate

Conformance SHALL be checked at runtime by `validate`, never encoded as a second, stricter type. Validity and lossiness are orthogonal: a conformant calendar may still carry `X-` or IANA extensions, so a no-extension type would name a useless category.

#### Scenario: A calendar carrying extensions
- GIVEN a conformant calendar with an `X-` property
- WHEN it is validated
- THEN validation passes and the extension is permitted

### Requirement: The validation walk

`validate` SHALL walk the whole component tree and report, per version: a property that version does not define, a value of a kind the property does not take, a parameter the property does not take, a property that appears more often than its cardinality permits, a property a component requires but does not carry, a component nested where it may not be, and a duration outside its grammar.

Absence and repetition are reported by different checks, because they know different things: a property's cardinality states how many times it may appear anywhere, while whether it is *required* depends on the component it sits in.

#### Scenario: A VEVENT with no UID
- GIVEN an iCalendar 2.0 `VEVENT` carrying no `UID`
- WHEN the calendar is validated
- THEN a required-property-absent problem is reported for `UID`

#### Scenario: A repeated single-valued property
- GIVEN a `VEVENT` carrying two `SUMMARY` properties
- WHEN the calendar is validated
- THEN a too-many problem is reported for `SUMMARY`

#### Scenario: A component nested where it may not be
- GIVEN a `VTIMEZONE` nested inside a `VEVENT`
- WHEN the calendar is validated
- THEN a nesting problem is reported

### Requirement: The contracts follow RFC 5545

A property's contract SHALL allow every value type and every parameter its RFC gives it, so a conformant calendar is never refused: `DTSTART`, `DTEND`, `DUE` and `RECURRENCE-ID` SHALL take `DATE` beside `DATE-TIME` and a `TZID`, and `RECURRENCE-ID` its `RANGE`; `EXDATE` and `RDATE` SHALL take a `TZID`; `ORGANIZER` SHALL take `CN`, `DIR`, `SENT-BY`, `LANGUAGE`, the RFC 7986 `EMAIL` and the RFC 6638 `SCHEDULE-AGENT`, `SCHEDULE-FORCE-SEND` and `SCHEDULE-STATUS`, as `ATTENDEE` does.

The rest of RFC 5545 section 3.8 and of the extensions the crate covers SHALL hold the same way: `ATTACH` takes an inline `BINARY` with its `FMTTYPE` and `ENCODING`, `FREEBUSY` its `FBTYPE`, `TRIGGER` an absolute `DATE-TIME` and its `RELATED`, and `DESCRIPTION` may repeat, as a `VJOURNAL` (RFC 5545 3.6.3) and a multilingual `VCALENDAR` (RFC 7986 5.2) carry several. `IMAGE` takes `BINARY`, `FMTTYPE`, `ENCODING`, `ALTREP` and `DISPLAY`, `CONFERENCE` its `FEATURE`, `LABEL` and `LANGUAGE`, `LINK` a `LANGUAGE`, `RELATED-TO` a `URI` (RFC 9253), `STRUCTURED-DATA` `BINARY` and `URI` with `FMTTYPE`, `SCHEMA` and `ENCODING`, and `STYLED-DESCRIPTION` a `URI` with `ALTREP`, `LANGUAGE`, `FMTTYPE` and `DERIVED`.

A contract MAY stay wider than its RFC: a property stating no parameters of its own takes `VALUE`, `LANGUAGE` and `ALTREP`, and a property's cardinality is one number for every component, so a `DESCRIPTION` repeated in a `VEVENT` passes. Narrowing either would refuse calendars the check passes today, and is a change of its own.

#### Scenario: A whole-day, zoned, organised event
- GIVEN a `VEVENT` carrying `DTSTART;VALUE=DATE`, `DTEND;TZID=Europe/Paris`, `EXDATE;TZID=Europe/Paris` and an `ORGANIZER` with `CN`, `SENT-BY` and `SCHEDULE-AGENT`
- WHEN the calendar is validated
- THEN no problem is reported

#### Scenario: A parameter the RFC still refuses
- GIVEN a `DTSTART` carrying `PARTSTAT`
- WHEN the calendar is validated
- THEN a parameter-not-allowed problem is reported for `DTSTART`

### Requirement: Durations are validated

A duration SHALL be checked against the RFC 5545 3.3.6 grammar exactly, wherever it is the value (`DURATION`, a relative `TRIGGER`, `REFRESH-INTERVAL`) or a parameter (the RFC 9253 `GAP`), and one outside it SHALL be reported with the property and the text as written: a week stands alone, a `T` opens a time part that names a unit, the units come upper case and in order, and a minute sits between an hour and a second. A period's duration (`FREEBUSY`, `RDATE;VALUE=PERIOD`) is not checked, the period value having no grammar check of its own yet.

Reading stays liberal: the same duration still reads as a length (see [decoded-model](./decoded-model.md)), so a caller computing an end is never left without one.

#### Scenario: A duration written the way calendars in the wild do
- GIVEN a `VEVENT` carrying `DURATION:P1H`, a `GAP=PT1H20S` and a `TRIGGER:-pt15m`
- WHEN the calendar is validated
- THEN a duration problem is reported for each, carrying the text as written
- AND each still reads as a number of seconds

### Requirement: Recurrence rules are validated

A decoded recurrence rule SHALL be checkable against RFC 5545 3.3.10, reporting every `BY` part the rule's frequency forbids, a `BYDAY` ordinal outside `MONTHLY` and `YEARLY`, a `BYDAY` ordinal at `YEARLY` beside `BYWEEKNO`, `BYSETPOS` with no other `BY` part, and `UNTIL` together with `COUNT`. A rule that passes SHALL earn the same proof a calendar does. Calendar validation SHALL reach the rules carried by `RRULE` and `EXRULE` when the `recur` feature is on.

Expansion stays liberal: a part validation reports is ignored when the rule is expanded, never applied and never refused.

#### Scenario: BYWEEKNO at a monthly frequency
- GIVEN `FREQ=MONTHLY;BYWEEKNO=3`
- WHEN the rule is validated
- THEN a forbidden-part problem is reported for `BYWEEKNO`
- AND expanding the same rule ignores the part entirely, as if it were absent

### Requirement: The spec dispatch answers for the property it is asked about

The runtime bridge from a property kind to its static spec SHALL carry the kind it describes, and that kind SHALL be the one it was dispatched from. Seventy hand-written arms over seventy files is exactly the shape a copy-paste slips through, and a marker answering for the wrong property would do so silently, for every caller.

Every property SHALL allow at least one value kind, and the kind in force with nothing declared SHALL be one of those it allows.

#### Scenario: A marker under the wrong arm
- GIVEN the dispatch from a property kind to its marker
- WHEN a marker's `KIND` does not match the arm it sits in
- THEN the mismatch is reported

### Requirement: The Valid proof

A calendar that passes validation SHALL earn an `IcalValid<Ical>` marker that only a validator can mint, and both `Ical` and `IcalValid<Ical>` SHALL convert back into a syntax tree.

### Requirement: The strict builder

`IcalPropBuilder` SHALL refuse, by returning an error, to construct a property the spec forbids for the target version: a disallowed value kind or a known parameter that property may not take. Extension parameters SHALL be allowed. Assembling a calendar by hand with no checks stays available as the escape hatch.

#### Scenario: A parameter the property forbids
- GIVEN a builder for a property whose spec excludes `LANGUAGE`
- WHEN `LANGUAGE` is set
- THEN the builder returns an error rather than the property

### Requirement: The contract is reachable without a parser

The property and component markers, their specs, the vtable dispatching the open kinds onto them, `Ical::validate` and `IcalPropBuilder` SHALL all be available with default features off. None of them parses anything, so none SHALL depend on the parser.

#### Scenario: A build with no parser
- GIVEN default features off
- WHEN a calendar built by hand is validated
- THEN it validates, and the crate pulls in no dependency
