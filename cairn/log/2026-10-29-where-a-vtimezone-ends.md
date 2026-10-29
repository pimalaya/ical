---
cairn: log
change: where-a-vtimezone-ends
date: 2026-10-29
---

# Where a VTIMEZONE ends

- `TZUNTIL` (RFC 7808 7.1) is `IcalPropKind::TzUntil`, read into `IcalTz::until`; applied only by `tzdb::zones`.
- `tzdb::zones` completes a `VTIMEZONE` forward from the database: past its `TZUNTIL`, or past its latest onset when no rule is endless and its last offset matches the database there. That guard keeps a sender's own zone (the +0300 Paris case) intact. RFC 5545 3.6.5 requires coverage of every instance, so a truncation breaks a MUST; the user approved the reversal.
- fablab_cottbus.ics left divergence.tsv. Tests in tests/recur_instant.rs and tests/tz.rs.

Capabilities moved: **tz**.
