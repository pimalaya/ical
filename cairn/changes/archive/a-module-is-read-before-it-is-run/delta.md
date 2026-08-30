---
cairn: delta
change: a-module-is-read-before-it-is-run
---

## MODIFIED Requirements

### Requirement: The merge is read in the order it runs

The three-way merge SHALL be arranged as the steps it performs: addressing a calendar, diffing a side against the base, deciding when two sides performed one act, judging whether a right-side act lands, and replaying the ones that do. Each step SHALL be its own module, and a function about one type SHALL be a method on it.

No behaviour changes: the same actions are reported, the same collisions are conflicts, and the merged calendar is byte for byte the one the previous arrangement produced.

### Requirement: A JSON codec is split by what it converts

The jCal and JSCalendar codecs SHALL each be split into an export half, an import half, and one module per conversion domain, with the conversions written as methods on the model types they convert.
