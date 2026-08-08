---
cairn: log
change: jscalendar
landed: 2026-08-08
---

# JSCalendar, RFC 8984

`src/jscalendar.rs` and the four modules under it, behind the `jscalendar` feature, which implies `jcal`: the escape hatch is written in jCal syntax, so the sibling codec is what reads and writes it.

## Which spec, exactly

Two documents describe this conversion and they do not agree. RFC 8984 is the published JSCalendar; draft-ietf-calext-jscalendar-icalendar (revision 23, June 2026) is the conversion rules, written against JSCalendar 2.0, itself still a draft. Taking the draft whole would produce a crate that speaks neither: it names `recurrenceRule` where RFC 8984 has `recurrenceRules`, and `calendarAddress` where RFC 8984 has `sendTo` and `replyTo`.

So the model is RFC 8984's and the conversion rules are the draft's wherever they are model-neutral, which is most of them and all of the interesting ones. The module header says so, and names the three members where the two part company, so nobody has to re-derive the decision.

## The escape hatch is the whole design

Everything else follows from one requirement: nothing is dropped, in either direction.

Going out, a property or component with no JSCalendar counterpart is kept whole under the object's `iCalendar` member, in jCal syntax (draft 5.1.1). A parameter left over after a property converts is kept in that member's `convertedProperties` record, keyed by the JSCalendar member the property became.

That record does more work than it looks. Several iCalendar properties share one JSCalendar member: `updated` is either a `DTSTAMP` or a `LAST-MODIFIED`, a link is any of `ATTACH`, `IMAGE` or `LINK`. Writing a record for every converted property would be exact and unreadable, so `hatch::default_name` holds what a reader is entitled to assume, and a record is written only where that assumption would be wrong. A `DTSTAMP` writes nothing; a `LAST-MODIFIED` writes its name.

Coming back, the mirror hatch applies. A member no iCalendar property holds becomes a `JSPROP` property carrying its JSON, located by a `JSPTR` parameter (draft 4.1.2), and the export grafts those back onto the object once everything else has converted. A collection key becomes a `JSID` parameter or property (draft 4.2.1, 4.1.1). That last one is not an optimisation: without it the keys are positions, positions depend on property order, and property order does not survive a conversion, so a second pass renumbered everything.

## What the corpus found

The test that mattered is three lines: convert every fixture that parses, convert it back, convert it again, and assert the two conversions are equal. A first conversion is allowed to normalise; a second one is not, or something the first pass could not express was quietly lost. It failed eight times, and each failure was a real defect:

- A `DTEND` written before its `DTSTART` fell into the hatch, because a duration cannot be computed before the start it is measured from. Ends now convert in a second pass over the properties.
- A calendar with two `CATEGORIES` lines kept only the last, because keywords are a set and the calendar level was replacing rather than accumulating.
- An all-day event lost its `TZID`, because RFC 5545 gives a `DATE` no time zone to be in, but the calendar that wrote one is where the object's zone came from.
- A `ROLE="REQ-PARTICIPANT"` became a role literally spelled with its quotes. Parameter quoting is syntax, and it comes off at the JSON boundary.
- A participant's `SCHEDULE-FORCE-SEND` and a link's `LINKREL` were written out and never read back.
- An occurrence that was both excluded by an `EXDATE` and overridden by a component produced a patch saying both, which RFC 8984 4.3.5 forbids. The exclusion wins: an occurrence that does not happen cannot also be described.
- An object with no `timeZone` was written back with a `Z`. No zone is floating time (RFC 8984 4.7.1), not UTC.
- An alarm with no `UID` was given one invented from its key, which then appeared in the hatch on the way out.

The sweep now covers 186 fixtures.

## Two changes outside the conversion

`X-OFFSET;VALUE=UTC-OFFSET:-0500` decoded to an undecoded value: the decoder consulted the property spec, an unknown name has none, and the declared `VALUE` was dropped on the floor. RFC 5545 3.2.20 does not say the parameter only applies to registered names. A name outside the vocabulary now still decodes as the kind it declares. Every corpus classification is unchanged.

The conversion cannot borrow from the JSON it reads, because applying a recurrence patch builds objects that exist nowhere in the input. Rather than allocate blindly inside the module, `into_owned` is now on `IcalValue`, `IcalParam`, `IcalProp`, `IcalComponent`, their name types and `Ical`, the counterpart of `Cow::into_owned` for a whole calendar. An offline replica reading a calendar out of a buffer it is about to drop wants the same method.

## What normalises

Three things, stated in the spec and in the module header, none of them recoverable from the JSON alone.

An `RRULE`'s `UNTIL` is UTC whenever `DTSTART` is, and RFC 8984 states it in the object's own zone. Shifting between the two needs the time-zone database, which this crate does not carry, so the wall-clock digits go across unshifted: exact for a floating or UTC object, off by that zone's offset for any other. A caller who wants better has `timezone`, which resolves offsets from the calendar's own `VTIMEZONE`.

A `DTEND` becomes a duration, so an event that ended in another zone than it started in comes back with the start's zone on both ends.

Ordering inside a component is lost. That is what a set of members means.

`cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features` and `cargo build --no-default-features` are green.

Capabilities moved: `jscalendar` (ADDED: JSCalendar conversion, nothing is dropped, recurrence folds and unfolds, what JSCalendar normalises); `decoded-model` (ADDED: a declared VALUE decides the kind known name or not, a decoded calendar can outlive its bytes).
