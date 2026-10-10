---
cairn: tasks
change: jscalendar-bis
---

# Tasks

- [x] List what draft-ietf-calext-jscalendarbis-22 changes from RFC 8984 for the conversion (Appendix A), and what draft-ietf-jmap-calendars asks of an event.
- [x] `IcalJscalendarVersion` and `Ical::to_jscalendar_as`; `to_jscalendar` stays RFC 8984.
- [x] Export at 2.0: `version`, entry `method`, `recurrenceRule`, `organizerCalendarAddress` and the joined owner Participant, `calendarAddress`, 2.0 roles, participant progress, retired members to the hatch, `display` set, 2.0 override restrictions.
- [x] Import: one liberal reader, 2.0 detected per object; the 2.0 organizer, roles, progress and method rules apply only to 2.0 objects.
- [x] Fixes found on the way: a `VLOCATION` Location keeps its hatch on the way back, a Participant with no address writes no empty `CALENDAR-ADDRESS`, an empty `REQUEST-STATUS` extra writes as none.
- [x] Tests: a zoned series with an excluded and a moved occurrence, organizer and attendees, an alert and a location round-tripping through 2.0; an event as Stalwart writes it; RFC 8984 reading unchanged; retired members; the corpus at 2.0 with no retired member set.
- [x] Fold into jscalendar, log, changelog, archive.
