---
cairn: tasks
change: where-a-vtimezone-ends
---

- [x] Check RFC 5545 3.6.5 and RFC 7808 7.1 wording on truncated definitions and `TZUNTIL`
- [x] Decide point 2 (forward completion) with the user; point 1 does not wait on it
- [x] Decode `TZUNTIL` (RFC 7808 7.1) into `IcalTz`
- [x] Past `TZUNTIL`, read a known name through the database
- [x] If decided: complete a finite `VTIMEZONE` forward with the database's later onsets
- [x] Tests: a `TZUNTIL` in the past, a truncated Berlin definition read in a later winter, a zone that abolished DST (nothing added)
- [x] Move fablab_cottbus.ics from divergence.tsv to agreement.tsv if it now agrees
- [x] Fold the spec, log, CHANGELOG
