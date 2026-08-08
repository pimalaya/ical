---
cairn: tasks
change: recurrence-set
---

- [x] A lazy k-way merge over sorted occurrence streams, deduplicating equal instants
- [x] Include DTSTART, every RRULE and every RDATE (date, date-time and period forms)
- [x] Subtract every EXDATE and every EXRULE
- [x] Apply RECURRENCE-ID overrides, replacing the matching instance
- [x] Apply RANGE=THISANDFUTURE, replacing the instance and everything after it
- [x] Build the set from a decoded component, VEVENT and VTODO alike
- [x] Cover the RFC 5545 combined examples as tests
