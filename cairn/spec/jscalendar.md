---
cairn: spec
capability: jscalendar
status: current
---

# JSCalendar

The JSCalendar JSON data model of a calendar, RFC 8984 and its 2.0 successor, behind the opt-in `jscalendar` feature, built on `jcal`. A `VCALENDAR` is a Group, a `VEVENT` an Event, a `VTODO` a Task; the boundary is a raw `serde_json::Value`, for the same reason jCal's is.

Unlike jCal, this is a re-modelling rather than a re-encoding: a `DTEND` is a duration, an `ATTENDEE` line is a Participant object, a `VALARM` is an Alert, and an overriding `VEVENT` is a patch inside the series it overrides rather than a component of its own. The conversion rules are those of draft-ietf-calext-jscalendar-icalendar, written against RFC 8984 by default, which is what other producers write, and against JSCalendar 2.0 (draft-ietf-calext-jscalendarbis-22) on request, which is what draft-ietf-jmap-calendars builds on.

### Requirement: JSCalendar conversion

A decoded calendar SHALL convert to and from the RFC 8984 data model, and to JSCalendar 2.0 on request. A `Group` is a whole calendar; a lone `Event` or `Task` is the calendar holding it, since that is what a JMAP calendar server hands out one object at a time. Only a root that is none of the three SHALL fail the import.

#### Scenario: A calendar of events

- GIVEN a `VCALENDAR` holding a `VEVENT` with a `DTSTART`, a `DTEND` and a `SUMMARY`
- WHEN it is converted to JSCalendar
- THEN the calendar is a `Group`, the event is an `Event` in its `entries`, and the event states a `start`, a `timeZone`, a `duration` and a `title`

#### Scenario: A lone event

- GIVEN a JSCalendar `Event` with no `Group` around it
- WHEN it is converted to a calendar
- THEN the calendar holds one `VEVENT`

### Requirement: Nothing is dropped

Everything the mapping cannot express SHALL be carried rather than dropped, and a second conversion SHALL change nothing a first one did not.

Exporting, a property or component with no JSCalendar counterpart is kept whole in the object's `iCalendar` member, in jCal syntax, and a parameter left over after a property converts is kept in that member's `convertedProperties` record. The same record names the property a member came from wherever more than one could have, so `updated` knows whether it was a `DTSTAMP` or a `LAST-MODIFIED`. A component stating both SHALL give `updated` its `DTSTAMP` and keep its `LAST-MODIFIED` whole in the hatch, whatever their order.

Importing, the mirror hatch applies: a member with no iCalendar counterpart becomes a `JSPROP` property holding its JSON, located by a `JSPTR` parameter, and a collection key becomes a `JSID` parameter or property so it survives the next conversion.

A Location or Participant carrying a hatch of its own SHALL read back as the `VLOCATION` or `PARTICIPANT` it came from, with what the hatch holds.

#### Scenario: A property outside the mapping

- GIVEN an event carrying a property RFC 8984 does not map
- WHEN it is converted to JSCalendar and back
- THEN the property returns from the escape hatch intact

#### Scenario: A member outside the mapping

- GIVEN an Event carrying a member no iCalendar property holds
- WHEN it is converted to a calendar and back
- THEN the member returns, having travelled as a `JSPROP` property

#### Scenario: The whole corpus

- GIVEN every fixture in the corpus that parses
- WHEN each is converted to JSCalendar, back to a calendar, and to JSCalendar again
- THEN the second conversion equals the first

### Requirement: Recurrence folds and unfolds

An overriding component SHALL fold into the series it overrides, as the patch that turns one into the other (RFC 8984 4.3.5), and SHALL unfold back into a component of its own. A component overrides a series when it carries a `RECURRENCE-ID` and another component of the same kind carries the same `UID`, no `RECURRENCE-ID` and an `RRULE`; without such a series it is a stand-alone instance and converts to an entry of its own.

An `RDATE` is an override with an empty patch and an `EXDATE` one whose patch says `excluded`. A date that is both excluded and overridden is excluded: an occurrence that does not happen cannot also be described, and RFC 8984 forbids the patch that would say both.

#### Scenario: An overriding component

- GIVEN a daily series and a component overriding one instance's start
- WHEN they are converted to JSCalendar
- THEN there is one entry, and its `recurrenceOverrides` holds one patch, naming only the start

#### Scenario: An instance with no series

- GIVEN a component carrying a `RECURRENCE-ID` whose series is not in the calendar
- WHEN it is converted
- THEN it is an entry of its own, stating its `recurrenceId`

### Requirement: What JSCalendar normalises

A round trip SHALL change a calendar in these ways and no other.

An `RRULE`'s `UNTIL` is stated in UTC beside a zoned `DTSTART` (RFC 5545 3.3.10), and RFC 8984 states it as a local date-time in the object's `timeZone` (4.3.3). Exporting, a UTC `UNTIL` of a zoned object SHALL be told on that zone's clock, and importing, the `until` of a zoned object SHALL be told back to UTC, read as RFC 5545 3.3.5 reads a written time (the earlier of a fold, the offset before a gap). The zone is the `VTIMEZONE` the calendar defines under the start's `TZID` (importing, one the Group's hatch carries), and behind the `tzdb` feature a zone the database knows, IANA or Windows named, a calendar's own definition completed from the database as `tzdb::zones` completes it; this holds for a lone object too. A UTC `UNTIL` in a fold's second pass reads back as the first, JSCalendar having no way to say which. A floating object's `UNTIL` SHALL stay floating and a UTC object's in UTC. An object shown without a time SHALL keep a `DATE` `UNTIL`, `T00:00:00` in JSCalendar. Without the `tzdb` feature, or for a zone neither the calendar nor the database defines, the wall-clock digits SHALL be carried across unshifted and written back without `Z`: exact for a floating or UTC object, off by the zone's offset exporting any other, and importing a local `UNTIL` that this crate's expansion compares on the start's clock as RFC 8984 does, though RFC 5545 wants UTC there.

A `DTEND` becomes a duration, so an event that ended in another time zone than it started in comes back with the start's zone on both ends.

Ordering is lost: of the lines inside a component, of the parameters on a line, of the parts of a recurrence rule (written back in RFC 5545 order) and of the items of a `CATEGORIES` (keywords are a set). An `EXDATE` or `RDATE` listing several dates comes back one line per date. A parameter stating what a property means with none (`VALUE` of the default type, `ROLE=REQ-PARTICIPANT`, `RELATED=START`) is not written back, and a parameter value is quoted only where RFC 5545 3.2 needs it. Folds and the case of names are the writer's.

#### Scenario: An event in a named zone

- GIVEN `DTSTART;TZID=Europe/Berlin` and `DTEND;TZID=Europe/Berlin` an hour and a half later
- WHEN the event is converted
- THEN it states `PT1H30M` and the zone once

#### Scenario: A UTC end of recurrence in a defined zone

- GIVEN a calendar defining a `VTIMEZONE` and a series starting in it whose `RRULE` ends `UNTIL=20270603T080000Z`
- WHEN it is converted to JSCalendar and back
- THEN the rule's `until` is the zone's `2027-06-03T10:00:00`, and the `RRULE` comes back ending `UNTIL=20270603T080000Z`

#### Scenario: A UTC end of recurrence in a zone only the database knows

- GIVEN the `tzdb` feature, no `VTIMEZONE`, and a series starting `DTSTART;TZID=Europe/Paris:20261203T100000` whose `RRULE` ends `UNTIL=20270603T080000Z`
- WHEN it is converted to JSCalendar and back
- THEN the rule's `until` is `2027-06-03T10:00:00`, and the `RRULE` comes back ending `UNTIL=20270603T080000Z`

#### Scenario: A local end of recurrence from a JMAP object

- GIVEN the `tzdb` feature and an Event, lone or in a Group with no hatch, in `Europe/Paris` whose rule's `until` is `2027-06-03T10:00:00`
- WHEN it is converted to iCalendar
- THEN the `RRULE` ends `UNTIL=20270603T080000Z`

#### Scenario: A local end of recurrence in a gap or a fold

- GIVEN the `tzdb` feature and a rule in `Europe/Paris` whose `until` is `2027-03-28T02:30:00`, or `2027-10-31T02:30:00`
- WHEN it is converted to iCalendar
- THEN the `RRULE` ends `UNTIL=20270328T013000Z` (the offset before the gap), or `UNTIL=20271031T003000Z` (the earlier of the fold)

#### Scenario: Floating, UTC and all-day ends of recurrence

- GIVEN a floating series ending `UNTIL=20270603T100000`, a UTC one ending `UNTIL=20270603T090000Z` and an all-day one ending `UNTIL=20270603`
- WHEN each is converted to JSCalendar and back
- THEN each `RRULE` comes back with its `UNTIL` as written

#### Scenario: Calendars as clients write them

- GIVEN each of tests/corpus/clients: an iOS series with `X-APPLE-*` properties and parameters, alarms, a `VTIMEZONE`, an `EXDATE` and a moved occurrence; an Exchange meeting with `X-MICROSOFT-*` properties, attendees, an alarm, a monthly rule ending in UTC, an `EXDATE` and an exception; a Google invitation with `X-NUM-GUESTS` parameters, two alarms, an `EXDATE` list and a moved occurrence; a Thunderbird task and all-day event with `X-MOZ-*` properties
- WHEN each is converted to JSCalendar, in RFC 8984 and in 2.0, and back
- THEN it comes back line for line once the orderings, lists, defaults, quotes and folds above are normalized, and a second conversion equals the first

### Requirement: JSCalendar 2.0

A calendar SHALL convert to JSCalendar 2.0 on request, RFC 8984 staying the default, and the import SHALL read both. An object is read as 2.0 when it or its Group states a `version` other than 1.0, or when it carries `recurrenceRule`, `organizerCalendarAddress`, `endTimeZone`, `mainLocationId` or a Participant's `calendarAddress`; the RFC 8984 names are read either way, and an RFC 8984 object reads as it always did.

Written as 2.0, the Group states `version` and no entry does (bis 3.1.2); the calendar's `METHOD` is every entry's `method`, since a 2.0 Group has none (bis 4.3, conversion 2.3.27). One `RRULE` is the `recurrenceRule`. `ORGANIZER` is the `organizerCalendarAddress` and an owner Participant addressed by its `calendarAddress` (conversion 2.3.29), which is the same Participant as the `ATTENDEE` of that address when the two agree on its name and email; the attendee's record then says the line was there, so it comes back. Roles have no default, `REQ-PARTICIPANT` is `required`, a `VTODO` attendee's progress is the Participant's beside an `accepted` status (conversion Table 13), and a Link's `display` is a set. An override patch never touches what 2.0 forbids (bis 3.3.4): a readdressed participant goes whole, and `relatedTo` may now change.

Written as 2.0, an object SHALL set no member 2.0 obsoletes or reserves (bis A.2.1, A.2.2). What such a member would have held stays in the escape hatch: a further `RRULE`, an `EXRULE`, `REQUEST-STATUS`, `COMPLETED`, a `VLOCATION`'s `DESCRIPTION`, a participant's `LANGUAGE` and `SCHEDULE-*` parameters, and a `JSPROP` carrying a retired member. A calendar holding no entry keeps its `METHOD` whole.

Read as 2.0, the organizer is `organizerCalendarAddress`, its Participant the first owner of that address; that Participant is no `ATTENDEE` when it holds the owner role alone, says nothing an `ORGANIZER` cannot and carries no record (conversion 3.6). The `ROLE` is the first of `chair`, `required`, `optional` and `informational`, else a role iCalendar has no word for, else `OWNER` for an owner that is not the organizer. An entry's `method` is the calendar's `METHOD`.

`endTimeZone` is not written, since a span between two zones needs the time-zone database; read, it rides a `JSPROP` and the `DTEND` stays in the start's zone.

#### Scenario: A scheduled series

- GIVEN a zoned weekly series with an `EXDATE`, a moved occurrence, an `ORGANIZER` that also attends, two more attendees, an alarm and a location
- WHEN it is converted to JSCalendar 2.0 and back
- THEN the Event holds `recurrenceRule`, `organizerCalendarAddress` and three Participants addressed by `calendarAddress`, none of the retired members, and the calendar comes back line for line

#### Scenario: An event a JMAP server wrote

- GIVEN a lone 2.0 Event with no `version`, its organizer's Participant holding its attendance, a monthly `recurrenceRule` and overrides
- WHEN it is read
- THEN it is one series and one moved occurrence, each with one `ORGANIZER` and an `ATTENDEE` per Participant, and no `JSPROP`

#### Scenario: The whole corpus at 2.0

- GIVEN every fixture in the corpus that parses
- WHEN each is converted to JSCalendar 2.0, back, and to 2.0 again
- THEN the second conversion equals the first, and neither sets a retired member

### Requirement: JSCalendar needs no parser

The `jscalendar` feature SHALL imply `jcal`, whose syntax carries the escape hatch, and neither SHALL imply `parser`. A JMAP client that never reads an iCalendar byte SHALL be able to depend on this crate for the conversion alone.
