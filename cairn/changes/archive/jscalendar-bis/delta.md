---
cairn: delta
change: jscalendar-bis
---

## ADDED Requirements

### Requirement: JSCalendar 2.0
In jscalendar. A calendar SHALL convert to JSCalendar 2.0 (draft-ietf-calext-jscalendarbis-22) on request, RFC 8984 staying the default, and the import SHALL read both, an object being 2.0 when it or its Group states a `version` other than 1.0 or when it carries `recurrenceRule`, `organizerCalendarAddress`, `endTimeZone`, `mainLocationId` or a Participant's `calendarAddress`. Written as 2.0, an object SHALL set no member 2.0 obsoletes or reserves; what such a member held stays in the escape hatch.

## MODIFIED Requirements

### Requirement: JSCalendar conversion
In jscalendar. The conversion rules are those of draft-ietf-calext-jscalendar-icalendar, written against RFC 8984 by default and against JSCalendar 2.0 on request.

### Requirement: Nothing is dropped
In jscalendar. A Location or Participant carrying a hatch of its own SHALL read back as the `VLOCATION` or `PARTICIPANT` it came from, with what the hatch holds.

### Requirement: A single value is not split on a separator it does not own
In decoded-model. A `REQUEST-STATUS` with no extra data SHALL encode as `code;description`, with no trailing `;`.

## REMOVED Requirements
