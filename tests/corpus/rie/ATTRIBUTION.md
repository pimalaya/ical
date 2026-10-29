# recurring-ical-events

The test calendars of [recurring-ical-events](https://github.com/niccokunzmann/python-recurring-ical-events), the Python library that expands the recurrences of a calendar into occurrences, and the oracle of the timeline corpus beside this folder.

- Source: `recurring_ical_events/test/calendars/*.ics`, taken from the v3.9.0 tag, the release the oracle runs. The one file there that is not a calendar (`date_exclude.txt`) is not taken.
- Licence: recurring-ical-events is distributed under the GNU Lesser General Public License 3.0 or later. These files keep that licence; they are test data vendored for interoperability testing, not part of the ical-rs library, which is MIT or Apache-2.0.
- Why: most of them come from bug reports, so they carry the recurrence shapes real producers write and the library had to learn to read: moved and deleted instances, `RANGE=THISANDFUTURE`, `SEQUENCE` revisions, truncated `VTIMEZONE`s, Windows zone names, `X-WR-TIMEZONE`, malformed rules.

Nothing here is edited. A calendar that answers differently from the oracle is triaged in tests/corpus/timeline, not changed.
