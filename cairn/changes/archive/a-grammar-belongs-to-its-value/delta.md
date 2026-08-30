---
cairn: delta
change: a-grammar-belongs-to-its-value
---

## ADDED Requirements

### Requirement: A duration and a UTC offset read as numbers

`IcalUtcOffset::seconds` SHALL return the offset in seconds east of UTC, and `None` for anything that is not the RFC 5545 3.3.14 form.

`IcalDuration::seconds` SHALL return the duration in seconds, a week counting as seven days, and `None` for anything that is not the RFC 5545 3.3.6 form. `IcalDuration::from_seconds` SHALL write a number of seconds back as a duration, in days and smaller units, such that reading it returns the number written.

Both types SHALL keep their raw text as the value, so nothing about byte-faithful round-tripping changes.
