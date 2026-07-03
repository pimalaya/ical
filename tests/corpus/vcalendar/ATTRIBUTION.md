# vCalendar 1.0 corpus

These `.ics` (`.vcs`) fixtures are modelled on the vCalendar 1.0 specification
(the versit / Internet Mail Consortium format that predates RFC 5545), which
uses `VERSION:1.0`, floating local date-times and the inline alarm properties
(`DALARM`, `AALARM`, ...) instead of nested `VALARM` components.

They are robustness inputs for the round-trip and decode harness, verifying the
library handles the legacy dialect alongside iCalendar 2.0.
