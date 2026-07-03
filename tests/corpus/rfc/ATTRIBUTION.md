# RFC corpus

These `.ics` fixtures are transcribed from, or closely modelled on, the example
calendars in [RFC 5545](https://www.rfc-editor.org/rfc/rfc5545) (Internet
Calendaring and Scheduling Core Object Specification, iCalendar). RFC text is
published by the IETF and may be reproduced.

They exercise the core component kinds (`VEVENT`, `VTODO`, `VJOURNAL`,
`VTIMEZONE` with `STANDARD`/`DAYLIGHT`, and a nested `VALARM`), recurrence
rules, and common parameters. They are robustness inputs for the round-trip and
decode harness, not golden output.
