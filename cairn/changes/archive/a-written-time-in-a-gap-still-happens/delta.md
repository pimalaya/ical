---
cairn: delta
change: a-written-time-in-a-gap-still-happens
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: The crossing to an instant follows the RFC's two answers
In tz, replacing "The crossing to an instant is named once". `instant` reads a generated time (3.3.10), naming nothing in a gap; `literal_offset` and `literal_instant` read a written `DATE-TIME` (3.3.5), the offset before a gap and the earlier in a fold; `IcalTz::local` gives the local time a zone shows at an instant.

## REMOVED Requirements
