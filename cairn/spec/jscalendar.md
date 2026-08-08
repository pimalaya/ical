---
cairn: spec
capability: jscalendar
status: current
---

# JSCalendar

The RFC 8984 JSON data model of a calendar, behind the opt-in `jscalendar` feature, built on `jcal`. A `VCALENDAR` is a Group, a `VEVENT` an Event, a `VTODO` a Task; the boundary is a raw `serde_json::Value`, for the same reason jCal's is.

Unlike jCal, this is a re-modelling rather than a re-encoding: a `DTEND` is a duration, an `ATTENDEE` line is a Participant object, a `VALARM` is an Alert, and an overriding `VEVENT` is a patch inside the series it overrides rather than a component of its own. The conversion rules are those of draft-ietf-calext-jscalendar-icalendar, read against the published RFC 8984 rather than its successor: where the draft names a member only JSCalendar 2.0 has, the crate writes RFC 8984's.

### Requirement: JSCalendar conversion

A decoded calendar SHALL convert to and from the RFC 8984 data model. A `Group` is a whole calendar; a lone `Event` or `Task` is the calendar holding it, since that is what a JMAP calendar server hands out one object at a time. Only a root that is none of the three SHALL fail the import.

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

Exporting, a property or component with no JSCalendar counterpart is kept whole in the object's `iCalendar` member, in jCal syntax, and a parameter left over after a property converts is kept in that member's `convertedProperties` record. The same record names the property a member came from wherever more than one could have, so `updated` knows whether it was a `DTSTAMP` or a `LAST-MODIFIED`.

Importing, the mirror hatch applies: a member with no iCalendar counterpart becomes a `JSPROP` property holding its JSON, located by a `JSPTR` parameter, and a collection key becomes a `JSID` parameter or property so it survives the next conversion.

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

Three things SHALL NOT survive a round trip unchanged, and no more.

An `RRULE`'s `UNTIL` is stated in UTC whenever `DTSTART` is, and RFC 8984 states it in the object's own time zone; shifting between the two needs the time-zone database, which this crate does not carry, so the wall-clock digits are carried across unshifted. That is exact for a floating or UTC object and off by that zone's offset for any other.

A `DTEND` becomes a duration, so an event that ended in another time zone than it started in comes back with the start's zone on both ends.

Ordering inside a component is lost, since a JSCalendar object is a set of members rather than a list of lines.

#### Scenario: An event in a named zone

- GIVEN `DTSTART;TZID=Europe/Berlin` and `DTEND;TZID=Europe/Berlin` an hour and a half later
- WHEN the event is converted
- THEN it states `PT1H30M` and the zone once
