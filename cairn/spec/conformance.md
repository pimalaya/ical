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

`validate` SHALL walk the whole component tree and report, per version: a property that version does not define, a value of a kind the property does not take, a parameter the property does not take, a property that appears more often than its cardinality permits, a property a component requires but does not carry, and a component nested where it may not be.

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
