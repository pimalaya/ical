# ical4j

Sample calendars from the [ical4j](https://github.com/ical4j/ical4j) test suite, the long-standing Java implementation of iCalendar.

- Source: `src/test/resources/samples/{valid,invalid,hcalendar}/*.ics`, each prefixed with the directory it came from, taken from the master branch.
- Licence: BSD 3-Clause, Copyright (c) 2012 Ben Fortuna. These files keep that licence.
- Why: ical4j collects calendars exported by real products (Outlook, Lotus Notes, Apple iCal, Google Calendar and others), and it is the only one of the three suites that keeps a directory of deliberately *invalid* ones. Those are the fixtures that exercise the recovering parse mode.

Nothing here is edited. A fixture that fails is a bug in ical-rs, not in the fixture. The `invalid_` prefix means ical4j's strict parser refuses it, not that ical-rs must.
