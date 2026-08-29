---
cairn: log
change: recurrence-conflict-scope
landed: 2026-08-29
---

# A recurrence conflict is about the ground the override stood on

The pairing of a series change with an instance change narrowed to what defines the recurrence set: the `DTSTART`, `DTEND`, `DURATION`, `RRULE`, `RDATE` and `EXDATE` of the series, or the series component itself. A room changed on the series is no longer reported against a summary changed on an override, so the reason now means what the module header always claimed it meant.

This reports strictly less than before and refuses nothing that was not refused before, a recurrence conflict having never decided anything.

Capabilities moved: merge (MODIFIED: A series and its instances).
