---
cairn: log
change: a-zone-change-contests-a-time-change
date: 2026-10-09
---

# A zone change contests a time change

The judge (tree/merge/judge.rs) let a change to `VALUE` contest the other side's change to the value, since the value is read by its declared type, and no other parameter. A side moving a floating start to 08:00 against one zoning it to `/example.org/Romance` at 10:00 therefore merged into a Romance 08:00, a time neither side wrote: `TZID` changes what the value means as much as `VALUE` does (RFC 5545 3.2.19).

`qualifies` names the parameters that do, after an audit of RFC 5545 section 3.2, RFC 7986 section 6 and the extensions the crate covers. How the value is read: `VALUE`, `TZID`, `ENCODING` (with the vCalendar 1.0 `CHARSET`), `LANGUAGE`, `FMTTYPE`. What it denotes: `RANGE` (which instances a `RECURRENCE-ID` names), `RELATED` (which end a `TRIGGER` counts from), `FBTYPE` (free or busy), `RELTYPE` and the RFC 9253 `LINKREL` (how a related component or a linked resource relates). Left out, as describing the thing a value names rather than the value: `ALTREP`, `CN`, `CUTYPE`, `DELEGATED-FROM`, `DELEGATED-TO`, `DIR`, `MEMBER`, `PARTSTAT`, `ROLE`, `RSVP`, `SENT-BY`, the RFC 6638 scheduling parameters, the RFC 7986 `DISPLAY`, `EMAIL`, `FEATURE` and `LABEL`, and the RFC 9253 `GAP`.

Any such parameter change now collides with a value-level action on the other side, whichever side made which, and the left side's line stands whole. The property tests' reference merge (tests/merge_props.rs) models the same rule.

Capabilities moved: merge.
