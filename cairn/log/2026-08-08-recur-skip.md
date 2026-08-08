---
cairn: log
change: recur-skip
landed: 2026-08-08
---

# RFC 7529 SKIP, for the Gregorian scale

RFC 7529 is two features in one document, and only one of them costs anything.

`SKIP` says what a rule means when the date it names does not exist: the 29th of February in a non-leap year, the 31st of a month with thirty days. It needs no calendar system, only the month lengths `recur::civil` already computes. Non-Gregorian calendar systems need the CLDR arithmetic the RFC itself sends implementers to ICU for, and a leap-month notation, and numeric ranges that widen because those calendars have longer years.

The crate parsed and stored both parts and expanded neither. That is worse than not implementing them: `RSCALE=GREGORIAN;FREQ=YEARLY;SKIP=FORWARD` on a leap-day start produced the leap-years-only answer that the parameter exists to prevent. We were handed the fix and produced the bug.

## What landed

`IcalRecurExpand::skipped_days` resolves the days a period's intended day of the month cannot land on. `BACKWARD` takes the last day the month does have, `FORWARD` the first day of the next one. Only a day the rule intends is resolved: each positive `BYMONTHDAY`, or the day the start supplies when no part selects one. A negative `BYMONTHDAY` counts back from the end of a month and so always names a day that exists, and in the Gregorian scale a month number always does too, which leaves the day of the month as the only thing that can be missing. That last observation is why this half is small: RFC 7529 defines skip behaviour for invalid months as well, and the Gregorian calendar has none.

A resolved day is not put back through the day-selecting parts. RFC 7529 4.1 resolves it after `BYMONTHDAY` has chosen it, and the whole point is to land on a date the rule did not choose. It can therefore fall outside the period that produced it, which is why the buffer is now sorted and deduplicated when a skip is in force: a forward resolution from February lands in March, after days March itself produced, and `BYMONTHDAY=30,31` with `BACKWARD` puts both on the 28th.

`SKIP` with no `RSCALE` beside it is now reported by validation (RFC 7529 4). It is a validation problem rather than a parse error, for the same reason `UNTIL` with `COUNT` is: a rule that says too much is still a rule.

## Why only this half

Both claims in the README and the docs are restated. An `RSCALE` naming another calendar system is parsed, carried through jCal and JSCalendar and the merge, and yields no occurrences. Yielding Gregorian dates under a Hebrew rule would be a wrong answer rather than a missing one, and the honest sentence is now the one written down.

The half that was built is the half that can be checked. libical resolves `SKIP` for the Gregorian scale with no ICU build, and it is already one of the two oracles behind the frozen recurrence corpus; python-dateutil does not parse `RSCALE` at all. So tests/corpus/recur/skip.tsv holds ten cases with libical's answers, frozen and replayed, and the expander matches all ten, the RFC's own worked table (4.3.4) included. That is one oracle rather than two, which is stated in the corpus header rather than glossed.

`cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features` (17 binaries), `cargo build --no-default-features` and `cargo deny check` are green.

Capabilities moved: `recurrence` (ADDED: SKIP resolves a day that does not exist, only the Gregorian scale is expanded, the SKIP corpus is replayed).
