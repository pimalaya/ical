# timeline

A frozen differential run of the timeline (`IcalRecurTimeline`, occurrences as instants over a window) against two oracles, recurring-ical-events 3.9.0 and ical.js 2.2.1, over the calendars of tests/corpus/rie, tests/corpus/icaljs and tests/corpus/libical (181 calendars).

- windows.tsv: the window each calendar is walked over, from a year before its earliest `DTSTART`, `DUE` or `RECURRENCE-ID` to three years after its latest. A calendar of tests/corpus/rie is named bare, one of the other folders by its path under tests/corpus (`icaljs/daily_recur.ics`).
- agreement.tsv: every series on which ical-rs and both oracles answered alike, with that answer: the earliest 200 occurrences starting in the window, start and end each. 924 series.
- split.tsv: every series on which the two oracles answered differently and ical-rs answered as one of them, with ical-rs's answer and the oracle it agrees with. A calendar one oracle refuses is an answer too, so its series land here when ical-rs agrees with the other. 82 series, 73 with recurring-ical-events and 9 with ical.js.
- divergence.tsv: the series where this crate deliberately answers as neither oracle does, each with the reason and what each oracle answers instead. 7 series. No series has both oracles agreeing against ical-rs.

None of these files is anyone else's work: the windows are chosen here and the answers are what the three implementations computed. The oracles are not vendored, only consulted, and their calendars are vendored in tests/corpus/rie, tests/corpus/icaljs and tests/corpus/libical under their own licences.

## Two oracles

recurring-ical-events (Python, over icalendar and dateutil) and ical.js (JavaScript, the engine of Thunderbird's and Nextcloud's calendars) share no code with each other or with this crate. A series all three answer alike is kept as it comes, a guard against regression. A series the oracles split on is kept with the side ical-rs takes, so a change of side is caught too; most splits are ical.js reading a calendar differently (a truncated `VTIMEZONE` before its first onset, a lenient time, an `RRULE` it refuses), each of which ical-rs answers as recurring-ical-events does. Every series ical-rs answers as neither oracle does was read by hand on 2026-10-29 and became either a fix to this crate or a line of divergence.tsv.

ical.js has no time zone database of its own: its consumers register one. The harness does as they do with the zones.json of @nextcloud/timezones 1.0.2 (format 2, tzdata 2025b), the IANA `VTIMEZONE`s and the Windows and legacy aliases that Thunderbird generates and Nextcloud ships: a `TZID` a calendar names and does not define is registered from it, the calendar's own `VTIMEZONE` always winning, and a `TZID` it does not know stays floating, which is ical.js's answer.

## Regenerating

harness/run.sh does the whole run and rewrites windows.tsv, agreement.tsv and split.tsv. It needs nothing installed: nix builds an interpreter carrying recurring-ical-events from nixpkgs, and fetches node from nixpkgs and the ical.js and @nextcloud/timezones npm tarballs pinned by hash. The series ical-rs answers as neither oracle does go to the work directory's disagreement.txt, which must hold exactly the series of divergence.tsv, plus every calendar an oracle refuses outright (27, among them the twelve divergence.tsv names).

The harness is seven files: src/main.rs (the windows, the ical-rs runner, and the cross), oracle_rie.py, oracle_icaljs.mjs, run.sh, header.txt and split_header.txt, and Cargo.toml. It is a detached cargo package, so it never joins the crate's own build, and run.sh builds it into the work directory.

## Never edit an expected value

A changed answer is a behaviour change. Fix the crate, or record the divergence with its reason; do not make the corpus agree with the code.
