---
cairn: change
id: repeated-parameter-ordinal
status: landed
created: 2026-08-29
---

# A parameter written twice is two parameters

## Why

`diff_prop` matches a base parameter to a side parameter by name with a first-match lookup, so a line carrying one parameter name twice compares its second occurrence against its first. They differ, a change nobody made is reported, both sides produce one, and they collide. `merge(base, base, base)` over `SUMMARY;RSVP=TRUE;RSVP=FALSE:Planning` reports two actions and a conflict. The fuzz target found it in 114 units, before mutating anything, on an ical4j fixture: real producers write these lines.

The apply side has the same lookup. A genuine change to the second `RSVP` is written onto the first, and a removal drops every parameter of that name rather than the one the action named.

RFC 5545 3.2 lets several parameters share a name where the parameter is a list, `DELEGATED-TO` and `MEMBER` among them, and producers repeat scalar ones too.

## What

Parameters are matched by name plus their position among the same-named parameters of the line, which is what properties already do. The field a parameter action occupies carries that position, so two actions on two different `RSVP`s do not collide, and the replay addresses the occurrence the action named.

The action shape is unchanged: the ordinal is what the merge routes on, not what it reports, and a report already carries the parameter itself.
