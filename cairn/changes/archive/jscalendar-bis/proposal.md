---
cairn: change
id: jscalendar-bis
status: landed
created: 2026-10-10
---

# JSCalendar 2.0

## Why

draft-ietf-jmap-calendars builds on draft-ietf-calext-jscalendarbis (JSCalendar 2.0), and a JMAP server speaking it does not read RFC 8984: Stalwart 0.16 refuses `recurrenceRules` and drops a participant addressed with `sendTo`, so a series written in RFC 8984 loses its rule and a write loses its attendees. The Pimalaya Android app renames three members at its JMAP boundary to get by. The conversion draft itself (draft-ietf-calext-jscalendar-icalendar-28) now converts against 2.0. The renaming belongs here, and it is more than three names.

## What

An `IcalJscalendarVersion` (`V1_0` for RFC 8984, `V2_0` for draft-ietf-calext-jscalendarbis-22) taken by a new `Ical::to_jscalendar_as`. `to_jscalendar` stays RFC 8984, which is what other producers write, so nothing breaks.

Writing 2.0 follows the conversion draft against the 2.0 model (Appendix A of the bis draft lists the differences):

- the Group states `version`; `METHOD` goes on every entry, since a 2.0 Group has no `method` (bis 4.3, conversion 2.3.27);
- one `RRULE` is `recurrenceRule`; a further `RRULE` and every `EXRULE` stay in the hatch (`recurrenceRules`, `excludedRecurrenceRules` obsolete, A.2.1);
- `ORGANIZER` is `organizerCalendarAddress` and an owner Participant, the same object as the `ATTENDEE` of that address when the two agree (conversion 2.3.29); a Participant's address is `calendarAddress` (`replyTo`, `sendTo` reserved, A.2.2.2);
- roles follow 2.0: no default `attendee`, `REQ-PARTICIPANT` is `required` (A.2.3); a `VTODO` attendee's progress is the Participant's (conversion Table 13);
- what 2.0 reserves or obsoletes is not written: `LANGUAGE` and the `SCHEDULE-*` parameters of a participant, `REQUEST-STATUS`, `COMPLETED` (`progressUpdated`), a `VLOCATION` description, all kept in the hatch instead;
- a Link's `display` is a set (A.2.3);
- an override patch never touches what 2.0 forbids, and may now carry `relatedTo` (bis 3.3.4).

Reading stays one liberal reader for both versions: it takes `recurrenceRule` and `recurrenceRules`, `calendarAddress` and `sendTo`, `organizerCalendarAddress` and `replyTo`, a `display` set or string, and skips `version`. An object reads as 2.0 when it or its Group says `version` 2.0 or it carries a 2.0-only member; only then do the 2.0 rules for the organizer, roles, participant progress and `method` apply, so RFC 8984 input reads exactly as before.
