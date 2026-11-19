---
cairn: log
change: until-on-the-series-clock
landed: 2026-11-19
---

# UNTIL on the series' clock

Landed [until-on-the-series-clock](../changes/archive/until-on-the-series-clock/proposal.md), asked for by calendula's JMAP backend (calendula cairn/changes/jmap-backend), which reads and writes every event through JSCalendar.

What 2026-11-18 (client-calendars-round-trip, commit 2d8b1cb on the server branch) already did: a UTC `UNTIL` was told on a `VTIMEZONE` the calendar defines, both ways, and `RRULE` converts after `DTSTART`. What it left: a zone only the database knows, which is every JMAP object (a JSCalendar object names its `timeZone` and carries no `VTIMEZONE`), and every lone object (`of_entry` read with no zones at all).

Now, behind `tzdb`, `until_zone` (src/jscalendar.rs) asks `tzdb::zone_in` (src/tzdb.rs): the calendar's own definition completed from the database as `tzdb::zones` completes it, else the database's zone, IANA or Windows named, built through the year of the `UNTIL`. Without `tzdb` it is the calendar's own definition, as before. `rule_to_json` and `rule_from_json` go through it; a gap reads at the offset before it and a fold at its earlier pass (RFC 5545 3.3.5), which is `IcalTzOffset::literal_instant`. A UTC `UNTIL` in a fold's second pass so comes back on the first pass: RFC 8984 has a local `until` and no way to say which.

Two import bugs found by the new tests and fixed: a floating object's `UNTIL` was written with a `Z`, and an all-day object's as `T000000Z`, both against RFC 5545 3.3.10 (same value type as `DTSTART`, floating beside floating). They now come back as written.

Without `tzdb`, kept as it was rather than refused: the conversion is infallible, and a Z-less local `UNTIL` is read by this crate's expansion (recur::set) on the start's clock, which is RFC 8984's own reading, so the import stays right for that reader even though RFC 5545 wants UTC there. The export of a UTC `UNTIL` in a zone the calendar does not define stays off by the zone's offset; a consumer that needs it exact enables `tzdb`. A test pins this.

tests/jscalendar_clients.rs split fixtures on CRLF only, but the four fixtures under tests/corpus/clients are committed with LF (core.autocrlf input), so both client tests failed on the committed tree with the whole fixture reported missing; the test now accepts either line ending. The fixtures are unchanged.

Tests (tests/jscalendar.rs): `tells_a_utc_until_on_a_zone_only_the_database_knows`, `tells_a_local_until_back_to_utc_on_a_zone_only_the_database_knows` (lone and grouped), `reads_a_local_until_in_a_gap_or_a_fold_as_a_written_time`, `tells_a_utc_until_in_a_fold_on_its_first_pass`, `keeps_the_until_of_a_floating_series_floating`, `keeps_the_until_of_an_all_day_series_a_date` failed against 2d8b1cb and pass now; `keeps_the_until_of_a_utc_series_in_utc`, `carries_the_until_of_a_zone_nothing_defines_unshifted` and `carries_the_until_of_an_undefined_zone_unshifted_without_the_database` pin behaviour that did not move. Nothing was timed.

Also fixed a garbled sentence in the src/tzdb.rs module doc ("between the two rules").

Left as found: a `DATE` `UNTIL` beside a timed `DTSTART` (invalid, reported by validation) is carried as local midnight, which is how the expansion reads it; `endTimeZone` stays as the spec says.

Capabilities moved: jscalendar.
