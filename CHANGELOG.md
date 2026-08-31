# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-08-31

### Added

- Added `IcalRecurExpand::in_zone` and `IcalRecurSet::expand_in_zone`, which drop the instances RFC 5545 3.3.10 forbids counting.

  A rule generating an instance at a local time the clock jumps over generates something that never happens, and the section is explicit that such an instance "MUST be ignored and MUST NOT be counted as part of the recurrence set". The zone enters expansion as a predicate on candidates and nothing more: occurrences stay civil, stepping stays total arithmetic, and a candidate in a gap is skipped before `COUNT` is spent, so a rule bounded by `COUNT=5` still yields five and runs one period further to do so. An expansion given no zone behaves exactly as it did. The filter sits on the rule streams alone: an `RDATE` names a date rather than generating one, so it is kept whatever the zone says of it.

- Added `IcalTzOffset::instant`, the instant a civil local time names under a resolution, in seconds since the Unix epoch. A gap names none, which is the RFC's own answer; a fold takes its earlier offset, which is a default the variant's fields still let a caller override.

- Added `IcalTz::is_gap`, `IcalTz::transitions`, `IcalTzTransition` and `IcalTzTransitions`, which materialise a zone's transitions rather than re-expanding every observance on every call. `IcalTz::resolve` answers by lookup over that list now, and a caller asking once per date, as a zoned expansion does, holds the list and grows it as the walk moves forward.

## [0.3.0] - 2026-08-30

### Added

- Added `IcalUtcOffset::seconds`, `IcalDuration::seconds` and `IcalDuration::from_seconds`, so a caller holding one of those values can read it as a number.

  Both kinds are kept as raw text, which is what byte-faithful round-tripping needs, and neither offered a way to read that text. Every consumer that needed one wrote its own parser instead: the time-zone layer, the JSCalendar import and the JSCalendar export each carried a private copy of one of the two grammars. Neither grammar carries a month or a year, so the answer is a plain number of seconds and no calendar is needed to reach it. The raw text is still the value.

- Added a property identity to the three-way merge, as a new `identity` field on `IcalPropPath`.

  A property that may occur more than once and whose value names a thing outside the calendar is now addressed by that whole value rather than by its position: `ATTENDEE` by its calendar user address, `ATTACH` by its URI or inline binary, `RELATED-TO` by the `UID` it points at, `CONFERENCE` and `IMAGE` by their URI. Two properties carrying different identities are never matched with each other, so changing an attendee address reads as a person leaving and another arriving rather than as a rename. Every other property keeps a position, and the field is `None` for those, as it is for a value two same-named siblings share, which tells neither of them apart.

### Changed

- Moved the property and component markers, with their specs, from the syntax layer to the decoded model. `ical::tree::prop::<name>::MARKER` is now `ical::prop::<name>::MARKER` and `ical::tree::component::<name>::MARKER` is `ical::component::<name>::MARKER`, with `IcalPropSpec`, `IcalPropCardinality` and the property vtable under `prop`, and `IcalComponentSpec` and the component vtable under `component`.

  A marker carried two things at once: the RFC contract of its property or component, and the projection that reads and edits one line or subtree of a byte tree. Only the second is syntax, and gating both on `parser` put the whole strict-out layer behind a parser it never touches. `IcalPropLens`, the projection, stays under `tree::prop::<name>`.

- Moved the strict-out layer to the crate root, following the contract it consults: `tree::ical::builder` is now `builder`, `tree::ical::validate` is `validator`, and `valid::IcalValid` moves into `validator` beside the check that mints it. `tree::ical` is gone.

- Renamed the `timezone` module to `tz`, and `IcalTimezone`, `IcalObservance` and `IcalOffset` to `IcalTz`, `IcalTzObservance` and `IcalTzOffset`, so the layer is scoped by the same three letters the RFC gives every property it reads.

  `IcalOffset` was the one that misled: it is not an offset but the answer to a resolution, which may be one offset, a gap the clock jumped over or a fold it repeated, and it sat beside `IcalUtcOffset`, which is the wire value. "Observance" stays, being RFC 5545 section 3.6.5's own word for one such rule.

- Changed the `jcal` feature to stop implying `parser`. Validation, the builder, jCal and JSCalendar are now all reachable with default features off, so `--no-default-features --features jscalendar` builds and pulls in `serde_json` alone.

- Changed the parameter codec to carry an escaping mode, which the parameter side never had.

  `IcalParamNode` gains an `escaper` field, stamped by the parser once `VERSION` is known exactly as a value node's already was, and read by every parameter decode. `IcalParam::encode` and `IcalParamLens::encode` take the target `Escaper`, mirroring `IcalProp::encode` and the `Codec` trait, so a parameter is written in the same version's rules it was read in. `Escaper` gains `has_param_encoding`, true for iCalendar 2.0 and false for vCalendar 1.0.

- Changed the value node accessors so a truncating read has to name the component it truncates at, replacing `decode_at`, `decode_scalar_at`, `decode_joined_at`, `decode_joined`, `decode_bytes_at`, `set_at` and `set_bytes_at`.

  Reading component zero looks like reading the value and is not: it stops at the first unescaped `;`. Almost every call site passed `0`, and that one shape produced four separate defects in two days across three crates. `decode`, `decode_list` and `decode_bytes` now read the whole value; `decode_component` and `decode_component_list` read one `;`-component and always spell out which. `decode_scalar_at` and `decode_bytes_at` are gone, having cut twice, at a `;` and then at a `,`, which no caller wanted. `set` and `set_bytes` replace the whole value, `set_component` and `set_component_bytes` name their slot, so a read and the write that follows it address the same thing. `IcalValueCursor` follows the same split: `text`, `bytes`, `list` and their setters address the whole value, `component` and `set_component` one slot.

### Removed

- Removed `IcalComponentLens`, an empty marker trait whose whole content was its `IcalComponentSpec` supertrait. `IcalCst::component`, `component_mut` and `components` bound on the spec directly.

- Removed organiser authority from the merge: the `right_speaks_for` field, the `Authority` conflict reason, and the RFC 5546 section 3.2 refusal they drove.

  The field named a side rather than a role, which forced the one caller that used it to put its local calendar on the right, while every other caller in the ecosystem puts local on the left. Removing it is what lets one convention hold everywhere: the left side is the one being merged into, its bytes are the merged bytes, and it wins a collision by default. The capability itself is worth having back, and the way back is a field that names its own side rather than a fixed one.

### Fixed

- Fixed a parameter value reading back with the double quotes RFC 5545 section 3.1 wraps it in.

  They are the grammar's delimiters, not content, so `ALTREP="cid:part1.0001@example.org"` now decodes to `cid:part1.0001@example.org`, in a lens read, in the decoded model, and in the jCal and JSCalendar exports. Encoding puts a pair back around a value carrying a `,`, a `;` or a `:`, so a parameter that needed quoting still gets it, and vCalendar 1.0, whose grammar has none, is unaffected. A calendar's own bytes are untouched: the quotes live on the syntax leaf, which round-tripping never reads through the codec.

  The merge no longer reports a change when one side merely re-quoted a parameter it left alone.

  **Breaking** for a caller that built a parameter with its own quotes: `IcalParam::AltRep(Cow::Borrowed("\"cid:...\""))` now means a value whose text starts and ends with a double quote, and goes out as `ALTREP="^'cid:...^'"`. Pass the value without them.

- Fixed the two structural parse errors calling a component a card, which is vCard's word and not this crate's.

- Fixed parameter value encoding, which read RFC 5545 section 3.3.11 text escapes into a parameter that has none, and never wrote RFC 6868 at all.

  Section 3.2 gives a parameter value no backslash escapes, which is the whole reason RFC 6868 exists. So a backslash a parameter legitimately carried, a Windows path in an `ALTREP` or an `X-` parameter, was eaten on the way in and could not be written back, while a real `^n`, `^^` or `^'` from a conforming producer reached the caller with its encoding showing. A parameter is now decoded and encoded by RFC 6868 section 3.1: `^n` is a newline, `^^` a caret, `^'` a double quote, any other caret sequence stays literal as section 3.1 requires, and a backslash is content in both directions. RFC 6868 updates RFC 5545 and no earlier specification, so the rules apply to iCalendar 2.0 alone and a vCalendar 1.0 caret stays a caret; `Escaper::has_param_encoding` is the switch, and a parameter node now carries its calendar's `Escaper` the way a value node already did.

- Fixed the merge reading two sides that wrote different bytes as one act, which dropped the difference without a word.

  Agreement was decided on the decoded actions, and a decode is not injective: `\N` and `\n` both unescape to a line break (RFC 5545 section 3.3.11), so two sides writing a value each way produced equal actions, the right side's act was skipped as already made, and no conflict was reported. Agreement is now byte equality at the granularity of the act itself, which the property and component additions already required. The one exception is a parameter the specification gives no order, `DELEGATED-FROM` and `DELEGATED-TO` (sections 3.2.4 and 3.2.5), `MEMBER` (section 3.2.11) and `FEATURE` (RFC 7986 section 6.3), whose values now compare as a set, so writing one list in two orders stays one act rather than becoming a conflict. The merged bytes are unchanged either way, since the left side keeps its value; what changes is that the divergence is reported.

- Fixed a list value being written back on a replay that changed nothing, which re-escaped items nobody edited.

- Fixed the merge comparing parameters decoded, which hid an edit the decode cannot see.

  A single-valued parameter decodes its first value alone, so `CN=Ada,Lovelace` and `CN=Ada,Byron` compared equal, the change was never reported, and the edit was dropped without a word. Parameters are now compared on their raw nodes, value by value, exactly as values already were, falling back to raw bytes across two calendars of different versions that share no decoding.

- Fixed every value read that was silently truncating a value it had no business splitting.

  RFC 5545 section 3.3.11 has a text value escape a `;` or a `,` it means literally, and section 3.3.13 gives a URI no escaping at all, so an unescaped separator is content. An `IcalText`, an `IcalTextList`, an `IcalDateTimeList`, an `IcalCalAddress`, an `IcalPeriod`, an `IcalBinary`, an `IcalBoolean`, an `IcalDate`, an `IcalDateTime`, an `IcalTime`, an `IcalDuration`, an `IcalFloat`, an `IcalInteger`, an `IcalUri` and an `IcalUtcOffset` now keep everything past their first `;`, and a `REQUEST-STATUS` description and its extra data keep the commas inside them rather than being cut at the first one. The cursor's `text`, `bytes` and `list` read the whole value, and their setters replace it, so reading a value and writing it straight back no longer leaves the tail of the old one behind.

  A URI was the worst of them, being cut on both sides: `ATTACH:data:text/plain;base64,QUFB` decoded to `data:text/plain` with the payload gone, and encoding then escaped the semicolon it had just used as a separator, so what did survive decoding did not survive its own round trip.

- Fixed five defects a green suite was hiding, by aligning the three-way merge with vcard-rs.

  The two crates state one merge contract and shared almost no implementation, and this side had drifted. A value was compared decoded rather than raw, and a text value decodes its first `;`-component alone, so `LOCATION:Room A;floor 2` edited to `floor 9` reported nothing and merged nothing. A list was diffed and replayed as a set rather than a multiset, so dropping one of two equal `CATEGORIES` items was invisible on the way in and took both on the way out. A replay target was corrected for the baseline side's removals but not for its additions, so a line that side inserted made every later edit land one property early, overwriting one and leaving the other stale. A property the baseline side removed and the other side edited twice came back once per edit rather than once. And a `VALUE` retyped on one side did not contest the other side's item edits, producing a property whose items contradict its own declared type (RFC 5545 section 3.8.5.2).

- Fixed a three-way merge reading one calendar address written in two cases as two people.

  A property identity was compared on the raw bytes, so `MAILTO:Ada@Example.com` and `mailto:ada@example.com` missed each other and an attendee who answered in a client that normalises its output was reported as one person leaving and another arriving. An identity is now lowercased for comparison, a URI scheme being case-insensitive (RFC 3986 section 3.1). Only the comparison normalises: a line still goes back out with the bytes the side that wrote it wrote.

- Fixed a `QUOTED-PRINTABLE` value ending on two `=` serializing with a line break in the middle of it.

  A line remembers what the parser resolved away as pieces indexed by offset, and the tokeniser and the line splitter each record one of the two `=`. The two lists were concatenated rather than merged, so the soft break went out before the dangling `=` it follows, and the reparse of that output joined the next line into the value. The pieces are now emitted in offset order.

- Fixed a three-way merge dropping, in silence, a change to a component that is not the first child of its parent.

  The lookup that read the replayed side numbered a component over all its parent's children rather than over its same-named ones, so a `DAYLIGHT` written after a `STANDARD` could not be found and the change was neither applied nor reported. A `VTIMEZONE` defining both observances was therefore unmergeable.

- Fixed a three-way merge dropping all but one of several removals from one group of same-named properties or components.

  Each removal named the position its target held in the base, and they were replayed in that order, so the first one renumbered the ones after it. Removing three attendees left one standing, and removing the first two kept the second and dropped the third. Removals now replay last and highest position first.

- Fixed a three-way merge writing one attendee's answer onto another, and inventing people who exist in no version of the calendar.

  A property was addressed by the position it held in the base, and that position was resolved against the merged calendar and against the replayed side, which are free to number the same group differently. Properties iCalendar identifies by what they name are now matched by that identity, and a position that remains is translated through the baseline side's own removals before it is used. Merging an untouched side with an edited one now returns the edited one exactly.

- Fixed a removal passing silently by the other side's work on what it removed.

  A whole-property removal met neither a parameter change nor a list-item change on that property, and a component removal met nothing nested inside it, so an attendee's reply or a reminder added to a deleted event disappeared with no collision reported. A removal now meets every action on what it takes away, at any depth, and the update wins as it always did, with granularity settling which side removes: dropping one parameter keeps the property.

- Fixed two sides that made the same change being reported as diverging.

  Collisions compared the field two actions occupied and never the values they carried, so `merge(base, x, x)` reported a conflict for every change `x` made, naming the same action on both sides. Two identical actions are now no collision, and merging two identical sides returns them unchanged and reports nothing.

- Fixed a calendar holding one `UID` twice, or one calendar address on two attendees, colliding with itself.

  Both components were matched against the same one on the other side, so the difference between the two duplicates was reported as a change each side made. Each component of one side is now matched with at most one component of the other, a value two same-named properties share no longer identifies either of them, and an addition, numbered in the side that added it, is no longer matched with an action naming a property the base held.

- Fixed both sides adding one property or component leaving two of them in the merged calendar.

  The right side's addition was appended beside the left side's rather than losing to it, so a `VEVENT` could come out with two `LOCATION` lines, which RFC 5545 forbids and this crate's own `validate` refuses. The merged calendar now holds the left side's alone and reports the collision.

- Fixed a property carrying one parameter name twice being reported as changed when nothing had changed.

  Parameters were matched by name with a first-match lookup, so a line's second `RSVP` compared against its first, and a change to the second was written onto the first while a removal dropped every parameter of that name. Parameters are now matched, addressed and removed by name plus their position among the same-named ones.

- Fixed a merge emitting a calendar its own parser refuses.

  A bare, envelope-less record holds `BEGIN` and `END` lines as properties, and copying one into a well-formed calendar spliced a structural keyword into the middle of it; a line copied from a truncated side carried no line ending and swallowed the line after it. The merge now treats `BEGIN` and `END` as the envelope on every side, and terminates what it copies.

- Fixed a recurrence conflict being reported for a change that cannot have moved an occurrence.

  Any change to a series paired with any change to one of its overrides, so a room change on the series was reported against a summary change on the override. Only a change to the `DTSTART`, `DTEND`, `DURATION`, `RRULE`, `RDATE` or `EXDATE` of a series, or to the series component itself, is reported against an instance now.

- Fixed a line being split inside a double-quoted parameter value.

  RFC 5545 section 3.2 lets a quoted parameter value carry a colon and a semicolon, and its own `DESCRIPTION;ALTREP="cid:part1.0001@example.org":Meeting notes` example does, but the head ended at the first colon anywhere and split on every semicolon. The parameter was cut in two and the rest of it read as the value, so a merge saw a parameter edit as an edit of the value and reported a collision that was not one. The bytes round-tripped throughout, which is why nothing showed. A head carrying an unbalanced quote still parses, at the first colon anywhere.

- Fixed a merge writing a line break into the head of a line it replayed a parameter onto.

  The parameter was re-encoded from its decoded form, and the pair is not a round trip: decoding resolves the value escapes, encoding puts none back, so a `LANGUAGE` holding a `\n` came out holding a real newline and the merged line lost its colon along with everything after the break. The replay now copies the parameter off the line the side that wrote it wrote, as it already copies a whole property, a value and a component.

- Fixed a value written into a vCalendar 1.0 calendar breaking the line it sits on.

  Versit escapes `;` alone, so a newline went out raw and the calendar no longer parsed. It reached anyone calling `set_text` on a 1.0 calendar, and the merge whenever a list item decoded from a 2.0 side was replayed onto a 1.0 one. A newline is now written as `\n`, the closest 1.0 can carry it, and reads back as those two characters.

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

  Components are matched by UID plus RECURRENCE-ID, properties by name then equality then position, and the merged calendar keeps the left side's bytes. Every action and every collision is reported, and a removal against an update keeps the update.

- Added opt-in content-decoding features, each backed by a `no_std` crate: `quoted-printable`, `base64` and `encoding`.

- Added the RFC 7265 jCal codec behind the `jcal` feature.

  It reads and writes the JSON representation through a raw `serde_json` value, resolving the type slot through the same property spec as the wire decoder.

- Added the RFC 8984 JSCalendar conversion behind the `jscalendar` feature.

  A VCALENDAR is a Group, a VEVENT an Event, a VTODO a Task, a VALARM an Alert, a DTEND a duration, and an overriding VEVENT a patch inside its series. What the mapping cannot express is carried in the object's iCalendar member or in a JSPROP property rather than dropped, following draft-ietf-calext-jscalendar-icalendar.

- Added the RFC 7953 availability components (VAVAILABILITY, AVAILABLE, BUSYTYPE), the RFC 9253 relationship vocabulary (LINK, REFID, CONCEPT, LINKREL, GAP) and the RFC 6638 CalDAV scheduling parameters (SCHEDULE-AGENT, SCHEDULE-FORCE-SEND, SCHEDULE-STATUS).

- Added the frozen recurrence corpus: 4,331 cases on which python-dateutil 2.9 and libical 3.0.20 agree, replayed with no Python and no C toolchain, plus the curated divergences and the harness that regenerates both.

- Added 190 real-world fixtures from the libical, ical4j and ical.js suites, swept for round-trip fidelity and cross-checked against calcard.

[0.4.0]: https://github.com/pimalaya/ical/compare/v0.3.0..v0.4.0
[0.3.0]: https://github.com/pimalaya/ical/compare/v0.2.0..v0.3.0
[0.2.0]: https://github.com/pimalaya/ical/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/ical/compare/root..v0.1.0
