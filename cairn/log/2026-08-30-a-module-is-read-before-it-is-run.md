---
cairn: log
change: a-module-is-read-before-it-is-run
date: 2026-08-30
---

# A module is read before it is run

Three files had outgrown a reading. `tree/merge.rs` held 1994 lines and 58 free functions, `jcal.rs` 892 and 26, `jscalendar/export.rs` and `import.rs` 1582 and 1217.

Length was the symptom. The fault was that a function taking one argument it is entirely about hides the type it belongs to, and that a file holding every step of an algorithm gives no order to read them in. The merge showed both: `key(cst, ordinal)`, `component_name(cst)`, `lines(cst)` and `find(cst, path)` were `IcalCst` methods written as functions, and nothing said which of the fifty-eight ran first.

The merge is now the five steps it performs. `node` addresses a calendar, walking it into components each carrying the `UID` and `RECURRENCE-ID` path that names it. `diff` matches a side against the base. `compare` says when two sides performed one act. `judge` decides whether the right side's act lands. `replay` puts the ones that land onto the left side's bytes. The header stopped explaining all five and points at them, and `merge()` reads as the pipeline it is. Eight free functions remain, each genuinely a function of its arguments alone.

jCal became `export`, `import`, `datetime`, `recur` and `json`, and its conversions became methods: `IcalProp::to_jcal`, `IcalComponent::from_jcal`, `IcalParam::scalar` and their siblings. The JSCalendar hatch, which had been reaching into a 900-line file for seven of them, now calls methods on the values it is converting, and the temporal helpers it shares are a named module rather than a pair of items picked out of the middle.

JSCalendar kept its top level, which was never the problem, and split its two large halves by JSCalendar domain: `temporal`, `participant`, `alert`, `place`, and for the export `descriptive`. The `Builder` is one type across five files, each holding the members of one domain.

Private structs stopped documenting every field. `Op`, `Slot`, `Verdict`, `Shift` and `Entry` say on the type what they used to say five times over.

No behaviour changed, and the merge property suite says so: the same 357 tests pass, byte for byte.

Capabilities moved: merge.
