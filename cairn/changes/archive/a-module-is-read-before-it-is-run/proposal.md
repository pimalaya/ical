---
cairn: change
id: a-module-is-read-before-it-is-run
status: landed
created: 2026-08-30
---

# A module is read before it is run

## Why

Three files had grown past what one reading can hold: `tree/merge.rs` at 1994 lines and 58 free functions, `jcal.rs` at 892 lines and 26, and `jscalendar/export.rs` and `import.rs` at 1582 and 1217.

Length alone is not the fault. The fault is that a free function taking one argument it is entirely about hides the thing it belongs to, and a file holding every step of an algorithm at once gives a reader no order to read them in.

The merge is the clearest case: `key(cst, ordinal)`, `component_name(cst)`, `lines(cst)`, `find(cst, path)` and a dozen more are all methods of `IcalCst` written as functions, and nothing in the file says which of the fifty-eight run first.

## What

Split each of the three by domain, into private submodules whose names are the steps of the thing they do, and attach every function that is about one type to that type.

The merge becomes `node` (addressing a calendar), `diff` (what one side changed), `compare` (when two sides did one thing), `judge` (whether the right side's act lands) and `replay` (putting it there). The module header stops explaining all five and points at them instead.

jCal becomes `export`, `import`, `datetime` (the temporal re-spellings), `recur` and `json` (the scalar vocabulary), with `IcalProp::to_jcal`, `IcalProp::from_jcal` and their siblings as methods on the types they convert.

JSCalendar keeps its `export` / `import` / `hatch` / `patch` shape, whose top level was never the problem, and splits the two large halves by JSCalendar domain: `temporal`, `participant`, `alert`, `place` and, for the export, `descriptive`.

A private struct or enum stops documenting every field: it is private, so a line on the type is enough. `insts` and other abbreviations are spelled out.

## Consequence

No behaviour changes. Every module is now shorter than its own explanation, and a reader following the merge meets the five steps in the order the algorithm runs them.
