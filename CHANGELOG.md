# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-08

### Added

- Added the version-agnostic decoded model, available without the `parser` feature.

  An `Ical` is a version plus the VCALENDAR-level properties and the components nested under it, an `IcalComponent` is recursive, and an `IcalProp` is a name, parameters and one value. Parameters and values are the open `IcalParam` and `IcalValue` enums, each with an `Unknown` arm so anything outside the model survives, alongside the value types `IcalText`, `IcalTextList`, `IcalBinary`, `IcalBoolean`, `IcalInteger`, `IcalFloat`, `IcalDate`, `IcalDateTime`, `IcalDateTimeList`, `IcalTime`, `IcalDuration`, `IcalPeriod`, `IcalUtcOffset`, `IcalCalAddress`, `IcalUri`, `IcalGeo`, `IcalRecur` and `IcalRequestStatus`. `into_owned` on each of them, on their name types and on `Ical` replaces every borrow with an allocation, so a decoded calendar outlives the bytes it was read from.

- Added the closed `IcalComponentKind`, `IcalPropKind`, `IcalParamKind`, `IcalValueKind` and `IcalVersion` vocabularies.

  Each reaches its wire spelling through `FromStr` and `Deref<str>`, and `IcalValue::kind` / `IcalParam::kind` recover the kind of an open value or parameter. `IcalPropName` holds either a known `IcalPropKind` or a verbatim unknown name, `IcalComponentName` does the same for a component, and an unrecognised or missing calendar version normalises to `IcalVersion::V2_0` in the decoded model, while byte-faithful round-tripping stays on the syntax tree. `IcalComponentKind::ALL`, `IcalPropKind::ALL`, `IcalValueKind::ALL` and `IcalParamKind::ALL` enumerate every known member.

- Added the byte-faithful syntax tree behind the `parser` feature (on by default).

  `IcalCst` parses bytes or text into a recursive component tree that reproduces the wire exactly, decodes onto the model, encodes back to a canonical tree, and edits one property in place through per-property lenses and byte-preserving cursors. A parsed calendar serializes back to its input byte for byte, its folds, its blank lines and its QUOTED-PRINTABLE soft breaks included: the tokeniser resolves a line's wire layout so every layer above sees one logical line, and records it on `IcalWire` so serialization lays it back out. An edit that changes a line's length drops that line's layout, since the recorded fold points no longer index the bytes they were taken against. `IcalCst::parse_many` round-trips a whole multi-calendar file, and `IcalCst::component` / `component_mut` / `components` walk the nested tree by lens marker (VEVENT, VTODO, VALARM, VTIMEZONE and the rest).

- Added `IcalCst::parse_recovering`, a parse that survives a malformed calendar.

  It keeps a line it cannot structure as an opaque item and carries on, closes a component left open at end of input, and reports every problem it worked around. The strict entry points are unchanged and stay the default.

- Added raw-byte value handling for foreign character sets.

  A property value is kept as bytes, so a value in a foreign CHARSET survives byte for byte, while a name or parameter must be UTF-8. `IcalValueCursor::bytes` and `set_bytes` are the byte escape hatch.

- Added the per-property `IcalPropSpec` and per-component `IcalComponentSpec` contracts, filled per RFC 5545 and its extensions.

  A property spec declares the versions it lives in, its cardinality, the value kinds and parameters it allows, and the value kind in force for a declared VALUE; a component spec declares the children it may nest and the properties it requires. The version axis is what lets validation report a property written in a version that does not define it: the vCalendar 1.0 alarm properties (AALARM, DALARM, MALARM, PALARM), RNUM and TZ belong to 1.0 alone, while every extension-RFC property and the RFC 5545 ones vCalendar 1.0 never had belong to 2.0 alone.

- Added the decoding rules for two cases the wire leaves ambiguous.

  A property declaring its own VALUE decodes as that kind whether or not its name is in the vocabulary (RFC 5545 3.2.20), and a single text value keeps an unescaped comma as data rather than truncating at it, since RFC 5545 3.3.11 says it should have been escaped and there is no list for it to separate.

- Added `Ical::validate`, an RFC 5545 conformance check over the decoded model.

  It walks the component tree and reports a property the version does not define, a value of a kind the property does not take, a parameter it does not take, a property that appears more often than its cardinality permits, a required property that is absent, a component nested where it may not be, and a recurrence rule that breaks RFC 5545 3.3.10. Extensions always pass. A calendar that passes earns `IcalValid<Ical>`, a proof only a validator can mint; both `Ical` and `IcalValid<Ical>` convert into an `IcalCst`.

- Added `IcalPropBuilder`, a version-aware, spec-driven builder for strict construction.

- Added the recurrence module, always available and dependency-free.

  `IcalRecurRule::parse` decodes RRULE text into typed parts, `IcalRecurExpand` is the lazy iterator yielding the occurrences a rule and a start denote per RFC 5545 3.3.10, and `IcalRecurRule::validate` reports every constraint of that section. Occurrences are civil `IcalRecurDateTime` values with no time zone, since RFC 5545 expands on the local wall-clock time of DTSTART. A part its frequency forbids is ignored whole, as if it were absent, and validation is what reports it. Parsing accepts a rule carrying both UNTIL and COUNT, since that pair is a constraint on meaning rather than on syntax, and validation reports it as `UntilWithCount`. An unsatisfiable rule ends rather than hanging, bounded by the year 9999 cap and by a budget of barren periods per occurrence.

- Added RFC 7529 SKIP for the Gregorian calendar scale.

  SKIP=BACKWARD moves a day of the month the month does not have onto its last day, SKIP=FORWARD onto the first day of the next one, and SKIP=OMIT (the default) drops it as RFC 5545 does. Only a day the rule intends is resolved, occurrences stay ordered and are emitted once, and `IcalRecurRule::validate` reports SKIP written without an RSCALE beside it. An RSCALE naming any other calendar system is still parsed, carried and converted, and still yields nothing.

- Added `IcalRecurSet`, the whole recurrence set a component denotes.

  It is DTSTART plus every RRULE and RDATE, minus every EXDATE and EXRULE, with RECURRENCE-ID overrides applied and RANGE=THISANDFUTURE shifting the tail. It walks as a lazy merge of sorted streams, and every occurrence carries both the identity the rules place it at and the start it actually happens at.

- Added the `timezone` module.

  `IcalTimezone::resolve` turns a civil date-time into a UTC offset from the VTIMEZONE the calendar carries, with no time-zone database and no new dependency, reporting the spring-forward gap and the fall-back fold rather than guessing one answer.

- Added `IcalMerge`, a three-way merge of two divergent calendars against their common base.

  Components are matched by UID plus RECURRENCE-ID, properties by name then equality then position, and the merged calendar keeps the left side's bytes: only the lines the right side changed are touched, so a folded line neither side edited stays folded. Every action each side took and every collision between them is reported. A field only one side touched is taken from that side, a field both touched keeps the left side's outcome, and a removal against an update keeps the update. A change to a series and a change to one of its instances both survive and are reported together; with `right_speaks_for` set, a change to a property only the organiser may set (RFC 5546 3.2) is refused and reported.

- Added opt-in content-decoding features, each backed by a `no_std` crate: `quoted-printable`, `base64` and `encoding`.

- Added the RFC 7265 jCal codec behind the `jcal` feature.

  It reads and writes the JSON representation of a calendar through a raw `serde_json` value, with the type slot resolved through the same property spec the wire decoder uses.

- Added the RFC 8984 JSCalendar conversion behind the `jscalendar` feature.

  A VCALENDAR is a Group, a VEVENT an Event, a VTODO a Task, a VALARM an Alert, an ATTENDEE a Participant, a DTEND a duration, and an overriding VEVENT a patch inside the series it overrides. What the mapping cannot express is carried rather than dropped: an iCalendar element with no JSCalendar counterpart goes in the object's iCalendar member, in jCal syntax, and a JSCalendar member with no iCalendar counterpart goes in a JSPROP property, following draft-ietf-calext-jscalendar-icalendar.

- Added the RFC 7953 availability components (VAVAILABILITY, AVAILABLE, BUSYTYPE), the RFC 9253 relationship vocabulary (LINK, REFID, CONCEPT, LINKREL, GAP) and the RFC 6638 CalDAV scheduling parameters (SCHEDULE-AGENT, SCHEDULE-FORCE-SEND, SCHEDULE-STATUS).

- Added the frozen recurrence corpus: 4,331 cases on which python-dateutil 2.9 and libical 3.0.20 agree with each other, replayed by the test suite with no Python and no C toolchain, plus a hand-curated corpus of the deliberate divergences and the harness that regenerates both.

- Added 190 real-world fixtures from the libical, ical4j and ical.js suites, swept for round-trip fidelity and cross-checked against calcard.

[unreleased]: https://github.com/pimalaya/ical/compare/v0.1.0..HEAD
[0.1.0]: https://github.com/pimalaya/ical/compare/root..v0.1.0
