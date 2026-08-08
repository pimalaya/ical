# libical

Real-world and hand-written calendars from the [libical](https://github.com/libical/libical) test suite, the reference C implementation of iCalendar.

- Source: `test-data/*.ics`, `src/test/*.ics` (prefixed `test_`) and `src/Net-ICal-Libical/test-data/*.ics` (prefixed `netical_`), taken from the master branch.
- Licence: libical is distributed under the Mozilla Public License 2.0 or the GNU Lesser General Public License 2.1, at the recipient's option. These files keep that licence; they are test data vendored for interoperability testing, not part of the ical-rs library, which is MIT or Apache-2.0.
- Why: libical is an independent implementation with three decades of vendor calendars behind it, so its fixtures carry the wire shapes real producers emit, malformed ones included.

Nothing here is edited. A fixture that fails is a bug in ical-rs, not in the fixture.
