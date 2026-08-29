# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added a property identity to the three-way merge, as a new `identity` field on `IcalPropPath`.

  A property that may occur more than once and whose value names a thing outside the calendar is now addressed by that whole value rather than by its position: `ATTENDEE` by its calendar user address, `ATTACH` by its URI or inline binary, `RELATED-TO` by the `UID` it points at, `CONFERENCE` and `IMAGE` by their URI. Two properties carrying different identities are never matched with each other, so changing an attendee address reads as a person leaving and another arriving rather than as a rename. Every other property keeps a position, and the field is `None` for those, as it is for a value two same-named siblings share, which tells neither of them apart.

- Added a collision preference to the three-way merge, as a breaking new field on `IcalMerge`.

  `prefer: IcalMergeSide` says which side's value the merged calendar carries where both sides changed one property to different things, apart from `left`, which now answers only whose untouched bytes survive. `IcalMergeSide::Left` is the default and the behaviour every merge had before. Every field of `IcalMerge` is public and callers build it as a struct literal, so the field has to be written out.

  The preference decides that case and no other: an update still beats a removal whichever side it came from, a property one side alone touched is still taken from that side, an untouched line still comes out byte for byte, and organiser authority is still judged on the right side alone, so a refusal does not depend on the preference.

### Fixed

- Fixed a three-way merge dropping, in silence, a change to a component that is not the first child of its parent.

  The lookup that read the replayed side numbered a component over all its parent's children rather than over its same-named ones, so a `DAYLIGHT` written after a `STANDARD` could not be found and the change was neither applied nor reported. A `VTIMEZONE` defining both observances was therefore unmergeable.

- Fixed a three-way merge dropping all but one of several removals from one group of same-named properties or components.

  Each removal named the position its target held in the base, and they were replayed in that order, so the first one renumbered the ones after it. Removing three attendees left one standing, and removing the first two kept the second and dropped the third. Removals now replay last and highest position first.

- Fixed a three-way merge writing one attendee's answer onto another, and inventing people who exist in no version of the calendar.

  A property was addressed by the position it held in the base, and that position was resolved against the merged calendar and against the replayed side, which are free to number the same group differently. Properties iCalendar identifies by what they name are now matched by that identity, and a position that remains is translated through the baseline side's own removals before it is used. Merging an untouched side with an edited one now returns the edited one exactly.

- Fixed a removal passing silently by the other side's work on what it removed.

  A whole-property removal met neither a parameter change nor a list-item change on that property, and a component removal met nothing nested inside it, so an attendee's reply or a reminder added to a deleted event disappeared with no collision reported. A removal now meets every action on what it takes away, at any depth, and the update wins as it always did, with granularity settling which side removes: dropping one parameter keeps the property.

- Fixed two sides that made the same change being reported as diverging.

  Collisions compared the field two actions occupied and never the values they carried, so `merge(base, x, x)` reported a conflict for every change `x` made, naming the same action on both sides. Two identical actions are now no collision, and merging two identical sides returns them unchanged and reports nothing under either preference.

- Fixed a calendar holding one `UID` twice, or one calendar address on two attendees, colliding with itself.

  Both components were matched against the same one on the other side, so the difference between the two duplicates was reported as a change each side made. Each component of one side is now matched with at most one component of the other, a value two same-named properties share no longer identifies either of them, and an addition, numbered in the side that added it, is no longer matched with an action naming a property the base held.

- Fixed both sides adding one property or component leaving two of them in the merged calendar.

  Under the right preference the winner was appended beside the loser rather than replacing it, so a `VEVENT` could come out with two `LOCATION` lines, which RFC 5545 forbids and this crate's own `validate` refuses. The winner now replaces the addition it beat, where it stood.

- Fixed a property carrying one parameter name twice being reported as changed when nothing had changed.

  Parameters were matched by name with a first-match lookup, so a line's second `RSVP` compared against its first, and a change to the second was written onto the first while a removal dropped every parameter of that name. Parameters are now matched, addressed and removed by name plus their position among the same-named ones.

- Fixed a merge emitting a calendar its own parser refuses.

  A bare, envelope-less record holds `BEGIN` and `END` lines as properties, and copying one into a well-formed calendar spliced a structural keyword into the middle of it; a line copied from a truncated side carried no line ending and swallowed the line after it. The merge now treats `BEGIN` and `END` as the envelope on every side, and terminates what it copies.

- Fixed a recurrence conflict being reported for a change that cannot have moved an occurrence.

  Any change to a series paired with any change to one of its overrides, so a room change on the series was reported against a summary change on the override. Only a change to the `DTSTART`, `DTEND`, `DURATION`, `RRULE`, `RDATE` or `EXDATE` of a series, or to the series component itself, is reported against an instance now.

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
