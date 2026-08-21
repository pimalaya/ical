# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-08-22

### Changed

- Moved the flattened re-exports onto their real module paths.

  The lens, spec, cardinality, node and cursor types now carry the module that owns them (`tree::prop::lens::IcalPropLens`, `tree::value::cursor::IcalValueCursor`), and `IcalRecurPart` and `IcalRecurRuleProblem` are reached through `recur::validate`.

## [0.1.0] - 2026-08-08

### Added

- Added the version-agnostic decoded model, available without the `parser` feature.

  An `Ical` is a version, the VCALENDAR-level properties and the nested components; `IcalComponent` is recursive, and `IcalProp` is a name, parameters and one value. `IcalParam` and `IcalValue` are open enums with an `Unknown` arm, and `into_owned` detaches a decoded calendar from the bytes it was read from.

- Added the closed `IcalComponentKind`, `IcalPropKind`, `IcalParamKind`, `IcalValueKind` and `IcalVersion` vocabularies.

  Each reaches its wire spelling through `FromStr` and `Deref<str>` and enumerates itself through `ALL`. An unknown name is kept verbatim, and an unrecognised or missing calendar version normalises to `IcalVersion::V2_0`.

- Added the byte-faithful syntax tree behind the `parser` feature (on by default).

  `IcalCst` parses into a recursive component tree, decodes onto the model, encodes back, and edits one property in place through per-property lenses and byte-preserving cursors. A parsed calendar serializes back byte for byte, folds, blank lines and QUOTED-PRINTABLE soft breaks included, since each line's wire layout is recorded on `IcalWire`; an edit that changes a line's length drops that layout.

- Added `IcalCst::parse_recovering`, a parse that survives a malformed calendar.

  It keeps what it cannot structure as an opaque item, closes a component left open at end of input, and reports every problem it worked around. The strict entry points stay the default.

- Added raw-byte value handling for foreign character sets.

  A property value is kept as bytes, so a foreign CHARSET survives byte for byte, while a name or parameter must be UTF-8; `IcalValueCursor::bytes` and `set_bytes` are the escape hatch.

- Added the per-property `IcalPropSpec` and per-component `IcalComponentSpec` contracts, filled per RFC 5545 and its extensions.

  A property spec declares the versions it lives in, its cardinality, and the value kinds and parameters it allows; a component spec declares the children it may nest and the properties it requires. The version axis is what lets validation report a vCalendar 1.0 property (AALARM, RNUM, TZ, ...) written in a 2.0 calendar, and the reverse.

- Added the decoding rules for two cases the wire leaves ambiguous.

  A property declaring its own VALUE decodes as that kind even when its name is unknown (RFC 5545 3.2.20), and a single text value keeps an unescaped comma as data rather than truncating at it (3.3.11).

- Added `Ical::validate`, an RFC 5545 conformance check over the decoded model.

  It walks the component tree and reports every violation for the version: undefined property, disallowed value kind or parameter, broken cardinality, missing required property, misnested component, malformed recurrence rule. Extensions pass, and a calendar that passes earns `IcalValid<Ical>`.

- Added `IcalPropBuilder`, a version-aware, spec-driven builder for strict construction.

- Added the recurrence module, always available and dependency-free.

  `IcalRecurRule::parse` decodes RRULE text into typed parts, `IcalRecurExpand` lazily yields the civil occurrences a rule and a start denote, and `IcalRecurRule::validate` reports every RFC 5545 3.3.10 constraint. Expansion ignores a part its frequency forbids, and an unsatisfiable rule ends rather than hanging.

- Added RFC 7529 SKIP for the Gregorian calendar scale.

  SKIP=BACKWARD moves a day the month does not have onto its last day, SKIP=FORWARD onto the first of the next one, and SKIP=OMIT drops it. An RSCALE naming any other calendar system is parsed and carried, but yields nothing.

- Added `IcalRecurSet`, the whole recurrence set a component denotes.

  It is DTSTART plus every RRULE and RDATE, minus every EXDATE and EXRULE, with RECURRENCE-ID overrides applied and RANGE=THISANDFUTURE shifting the tail. Every occurrence carries both the identity the rules place it at and the start it happens at.

- Added the `timezone` module.

  `IcalTimezone::resolve` turns a civil date-time into a UTC offset from the calendar's own VTIMEZONE, with no time-zone database, reporting the spring-forward gap and the fall-back fold rather than guessing.

- Added `IcalMerge`, a three-way merge of two divergent calendars against their common base.

  Components are matched by UID plus RECURRENCE-ID, properties by name then equality then position, and the merged calendar keeps the left side's bytes. Every action and every collision is reported: a removal against an update keeps the update, and with `right_speaks_for` set, an attendee's change to an organiser-owned property is refused (RFC 5546 3.2).

- Added opt-in content-decoding features, each backed by a `no_std` crate: `quoted-printable`, `base64` and `encoding`.

- Added the RFC 7265 jCal codec behind the `jcal` feature.

  It reads and writes the JSON representation through a raw `serde_json` value, resolving the type slot through the same property spec as the wire decoder.

- Added the RFC 8984 JSCalendar conversion behind the `jscalendar` feature.

  A VCALENDAR is a Group, a VEVENT an Event, a VTODO a Task, a VALARM an Alert, a DTEND a duration, and an overriding VEVENT a patch inside its series. What the mapping cannot express is carried in the object's iCalendar member or in a JSPROP property rather than dropped, following draft-ietf-calext-jscalendar-icalendar.

- Added the RFC 7953 availability components (VAVAILABILITY, AVAILABLE, BUSYTYPE), the RFC 9253 relationship vocabulary (LINK, REFID, CONCEPT, LINKREL, GAP) and the RFC 6638 CalDAV scheduling parameters (SCHEDULE-AGENT, SCHEDULE-FORCE-SEND, SCHEDULE-STATUS).

- Added the frozen recurrence corpus: 4,331 cases on which python-dateutil 2.9 and libical 3.0.20 agree, replayed with no Python and no C toolchain, plus the curated divergences and the harness that regenerates both.

- Added 190 real-world fixtures from the libical, ical4j and ical.js suites, swept for round-trip fidelity and cross-checked against calcard.

[unreleased]: https://github.com/pimalaya/ical/compare/v0.2.0..HEAD
[0.2.0]: https://github.com/pimalaya/ical/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/ical/compare/root..v0.1.0
