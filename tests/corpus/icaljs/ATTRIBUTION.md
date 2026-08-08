# ical.js

Sample and parser fixtures from [ical.js](https://github.com/kewisch/ical.js), the JavaScript implementation that Thunderbird ships.

- Source: `samples/*.ics`, `samples/timezones/**/*.ics` (prefixed `timezone_`) and `test/parser/*.ics` (prefixed `parser_`), taken from the main branch.
- Licence: Mozilla Public License 2.0. These files keep that licence; they are test data vendored for interoperability testing, not part of the ical-rs library, which is MIT or Apache-2.0.
- Why: the `parser_` fixtures are minimal, single-feature calendars that pin one wire detail each (a value type, an escape, a fold, a blank line), which makes a failure easy to localise. The samples are whole calendars from Google, Apple and Thunderbird.

Nothing here is edited. A fixture that fails is a bug in ical-rs, not in the fixture.
