---
cairn: tasks
change: vtimezone-offsets
---

- [x] Read a VTIMEZONE into its observances (STANDARD and DAYLIGHT, each with its start, offsets and rule)
- [x] Expand each observance's rule lazily to find the one in force at a civil time
- [x] Return the resolved offset for an unambiguous local time
- [x] Report a skipped local time (spring-forward gap) rather than guessing
- [x] Report an ambiguous local time (fall-back fold) with both offsets
- [x] Drive the tests from the RFC 5545 timezone fixture
