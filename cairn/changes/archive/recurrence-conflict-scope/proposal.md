---
cairn: change
id: recurrence-conflict-scope
status: landed
created: 2026-08-29
---

# A recurrence conflict is about the ground the override stood on

## Why

The module header says a recurrence conflict is one side changing the series, its `RRULE`, `RDATE`, `EXDATE` or start, while the other changes one instance of it. `across_the_series` reads only the two component paths, so any change to a series pairs with any change to an override. A room changed on the series is reported against a summary changed on the override, and nothing there moved the ground the override stood on.

The code follows the spec, which says only that the two are reported together, so the header is the document that is wrong. The header is the crate's architecture entry point, and a reader calibrates how much attention a recurrence conflict deserves from it. Today a caller cannot tell a rule change that may have orphaned an override from a room change that cannot have.

## What

The check narrows to what defines the recurrence set: a change on the series to its `DTSTART`, `DTEND`, `DURATION`, `RRULE`, `RDATE` or `EXDATE`, or the addition or removal of the series itself. The instance side stays unrestricted, since any change to an override is a statement about an occurrence the rule may have moved.

The spec gains the restriction the header already claimed, so the reason means something. This refuses nothing that was not refused before: a recurrence conflict has never decided anything, it only reports.

## Judgement call the owner should review

This reports strictly less than before. A caller that treated any series-and-override pair as suspicious will now be told only about the pairs that can actually have orphaned an override.
