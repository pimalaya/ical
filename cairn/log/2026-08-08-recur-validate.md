---
cairn: log
change: recur-validate
landed: 2026-08-08
---

# Validation, made to say what it claimed

Writing the spec down caught the crate out. `cairn/spec/conformance.md`, the CHANGELOG and the builder's own documentation all said validation checked value kinds, version-aware parameters, cardinality and component nesting. `validate_prop` checked one thing: whether the property existed in the version. Everything else was a table nobody read.

`Ical::validate` now performs all of it. A value of a kind the property does not take, a known parameter it does not take, a property that appears more often than its cardinality permits, and a component nested where its parent's spec does not allow it, each with an error variant naming what and where. Extensions still pass at every step, since validity is a predicate over the known vocabulary and an extension is outside it by construction.

The cardinality axis was empty: not one of the 70 property specs overrode it, so every property was repeatable and the check would have been dead code. The 36 properties RFC 5545 allows at most once per component now say so. Absence and repetition stay separate checks on purpose: a property's cardinality states how many times it may appear anywhere, while whether it is *required* depends on the component it sits in, which is what `required_props` knows.

`IcalRecurRule::validate` is the rule-level half, in `recur::validate`: every `BY` part the frequency forbids, a `BYDAY` ordinal outside `MONTHLY` and `YEARLY`, an ordinal at `YEARLY` beside `BYWEEKNO`, `BYSETPOS` with nothing to pick from, and `UNTIL` together with `COUNT` (which parsing already refuses, but a rule built by hand can still carry). Calendar validation reaches the rules on `RRULE` and `EXRULE` when the `recur` feature is on. `Valid<T>` moved to a new dependency-free `valid` module so both validators can mint the same proof without either depending on the other's feature.

Then the same exercise caught a second thing, and this one was a bug. The settled requirement says a `BY` part at a frequency the RFC forbids it at is *ignored*, and the expander did not do that. `BYYEARDAY` was applied as a limit at every frequency, `BYMONTHDAY` at `WEEKLY` likewise, and `BYWEEKNO` outside `YEARLY` was dropped as a limit but still counted as "a part selects the day", which suppressed the rule that a monthly period takes its day from `DTSTART`. So `FREQ=MONTHLY;BYWEEKNO=3` expanded to *every day of the month* rather than to the day the start names. Expansion now ignores such a part whole, which is the behaviour the spec always described, and the requirement is restated to cover every part rather than the two it happened to name. The 42 RFC 5545 worked examples are unaffected.

The differential corpus was regenerated against the fixed expander before being frozen, so what is committed is the current answer, not the old one.

Capabilities moved: `conformance` (MODIFIED: the validation walk; ADDED: recurrence rules are validated); `recurrence` (MODIFIED: a part its frequency forbids is ignored, replacing the narrower statement about `BYWEEKNO` and `BYYEARDAY`).
