---
cairn: log
change: client-calendars-round-trip
landed: 2026-11-18
---

# Client calendars round-trip

Landed [client-calendars-round-trip](../changes/archive/client-calendars-round-trip/proposal.md), asked for by Pimalaya Server, which converts every stored calendar between iCalendar and JSCalendar.

What already held: the escape hatch (the `iCalendar` member, `convertedProperties`, `JSPROP`) carried every unknown property, parameter and component both ways, and the corpus sweeps held the JSON side to a fixpoint. What nothing checked was the iCalendar side line for line on calendars as clients write them.

Four fixtures under tests/corpus/clients, written in the shape each client produces: an iOS series (`X-APPLE-STRUCTURED-LOCATION` with quoted `X-` parameters, `X-APPLE-*` properties, two alarms with their own `X-` properties, a `VTIMEZONE`, an `EXDATE` and a moved occurrence), an Exchange meeting (`METHOD:REQUEST`, a Windows-named `VTIMEZONE`, organizer and attendees, `X-NUM-GUESTS`, a monthly rule ending in UTC, an `EXDATE`, an exception, `X-MICROSOFT-*` lines), a Google invitation (an `EXDATE` list, an `EMAIL` and a `DISPLAY` alarm, a moved occurrence, `X-GOOGLE-CONFERENCE`) and a Thunderbird task and all-day event (`X-MOZ-*`, a `RELATED=END` alarm). tests/jscalendar_clients.rs converts each to JSCalendar, in RFC 8984 and in 2.0, and back, and holds the result line for line to the fixture once normalized, then a second pass to the first Group.

It found two losses, both fixed:

- A component stating `DTSTAMP` and `LAST-MODIFIED` kept only the one converted last in `updated`, dropping the other. `DTSTAMP` now holds `updated` and `LAST-MODIFIED` stays whole in the hatch, whatever their order (`Builder::stamped` in src/jscalendar/export.rs). The iOS, Google and Thunderbird fixtures carry both.
- A UTC `UNTIL` on a zoned series came back with its digits unshifted and no `Z`, so the Exchange series ending `20270603T080000Z` (10:00 local) lost its last occurrence. The export now tells a UTC `UNTIL` on the `VTIMEZONE` the calendar defines under the start's `TZID`, and the import tells a local one back to UTC on the `VTIMEZONE` the Group's hatch carries (`rule_to_json`, `rule_from_json`, `utc_text`; `IcalTz::told` is now crate-visible). `RRULE` and `EXRULE` convert after `DTSTART`, so the zone is known whatever the line order. Without such a `VTIMEZONE` the old behaviour stays.

What the check normalizes, now stated in the spec: line, parameter, rule part and `CATEGORIES` order, an `EXDATE` list split one date a line, a parameter stating the default (`VALUE` of the default type, `ROLE=REQ-PARTICIPANT`, `RELATED=START`) not written back, quotes only where needed, folds and name case. The spec's former "three things and no more" understated this; the requirement now lists every one.

Tests: tests/jscalendar_clients.rs (two, both versions), and in tests/jscalendar.rs `keeps_last_modified_beside_the_dtstamp_holding_updated` and `tells_a_utc_until_on_the_zone_the_calendar_defines`. The fixture sweep failed on both losses before the fixes (`DTSTAMP` missing from the iOS, Google and Thunderbird fixtures, the Exchange `UNTIL` without its `Z`). Nothing was timed; the parse path is untouched.

Left as found: a zone only the bundled database knows (`tzdb`) is not used for `UNTIL`; `endTimeZone` and the `DTEND` zone stay as the spec says.

Capabilities moved: jscalendar.
