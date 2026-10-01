---
cairn: log
change: vtimezone-from-the-database
landed: 2026-10-01
---

# A VTIMEZONE can be synthesized from the database, opt-in

io-gcal synthesized the VTIMEZONE of the zones a Google event names, from jiff's bundled database, and the Microsoft Graph event projection in io-msgraph needs the same. Rather than a second copy, the module moved here as `tzdb` (src/tzdb.rs), behind a `tzdb` feature pulling jiff with `alloc` and `tzdb-bundle-always`, so the bare core stays dependency-free and offset resolution keeps working from the calendar's own VTIMEZONE alone. Its seven tests came along unchanged.

Capability moved: **tz** (a VTIMEZONE can be synthesized from the database, opt-in).
