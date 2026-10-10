---
cairn: log
change: jscalendar-bis
date: 2026-10-10
---

# JSCalendar 2.0

draft-ietf-jmap-calendars builds on JSCalendar 2.0 (draft-ietf-calext-jscalendarbis), and a JMAP server speaking it does not read RFC 8984: Stalwart 0.16 refuses `recurrenceRules` and drops a participant addressed with `sendTo`. The conversion draft (draft-ietf-calext-jscalendar-icalendar-28) converts against 2.0 too.

`IcalJscalendarVersion` (`V1_0`, `V2_0`) and `Ical::to_jscalendar_as` landed in src/jscalendar.rs; `to_jscalendar` stays RFC 8984, so nothing breaks. The version rides the export `Builder`. At 2.0 the Group states `version` and hands its `method` to every entry; `rule` keeps one `recurrenceRule` and any further `RRULE` in the hatch; `ORGANIZER` converts first, as `organizerCalendarAddress` and an owner Participant, which the `ATTENDEE` of that address joins when the two agree on name and email, the attendee's record then forced (`IcalHatch::note_always`) so the line comes back. Roles drop the default `attendee` and spell `REQ-PARTICIPANT` `required`; a task attendee's progress moves onto the Participant. `EXRULE`, `REQUEST-STATUS`, `COMPLETED`, a `VLOCATION`'s `DESCRIPTION`, a participant's `LANGUAGE` and `SCHEDULE-*` parameters, and a `JSPROP` carrying a retired member stay in the hatch. A Link's `display` is a set. `patch::diff` takes the version: 2.0's list of unpatchable members lets `relatedTo` through, and a readdressed participant goes whole.

The import stays one reader: `is_v2` says per object whether 2.0 rules apply (a `version` other than 1.0 on the object or its Group, or a 2.0-only member), and the 2.0 names are read beside RFC 8984's. Only a 2.0 object reads its organizer from `organizerCalendarAddress`, drops the organizer's Participant when it says nothing an `ORGANIZER` cannot (conversion draft 3.6), maps roles and participant progress the 2.0 way, and hoists `method` to the calendar's `METHOD`.

Three fixes came out of the round trips: a Location carrying a hatch reads back as its `VLOCATION`, hatch included, rather than as a bare `LOCATION`; a Participant with no address writes no empty `CALENDAR-ADDRESS`; and an empty `REQUEST-STATUS` extra encodes as none rather than a trailing `;` (tree/value/request_status.rs).

`endTimeZone` is still not written: the span between two zones needs the time-zone database.

Capabilities moved: jscalendar, decoded-model.
