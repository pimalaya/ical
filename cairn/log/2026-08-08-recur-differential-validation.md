---
cairn: log
change: recur-differential-validation
landed: 2026-08-08
---

# Differential validation of the recurrence layer

The `recur` feature was challenged against two independent implementations rather than against its own reasoning: python-dateutil 2.9 (the de-facto reference, and the ancestor of rrule.js and most of the JS and Python calendar stacks) and libical 3.0.20 (an independent C implementation). The corpus was 9,245 generated (rule, `DTSTART`) pairs, twelve occurrences each: every frequency crossed with every `BY` part, twelve interacting part pairs, six thousand seeded random combinations of two to four parts, eight starts chosen for their edges (leap day, month end, year end, Mondays), plus the `UNTIL` and `COUNT` bounds. The parser was swept over 191 real-world `.ics` files taken from the libical, ical4j and ical.js test suites.

The harness lived in a scratch directory and is not committed. Reproducing it means: a Rust binary that reads a two-column TSV of start and rule and prints twelve occurrences per line; a Python script doing the same through `dateutil.rrule.rrulestr`; a C driver doing the same through `icalrecur_iterator_next`. Both oracles need a per-case alarm, since a rule neither can satisfy walks to year 9999 one tick at a time.

Outcome, on the 3,590 cases where the two oracles agree with each other: ical-rs matched 3,573 before, and matches 3,589 after. Three defects were found and fixed.

The first was a panic reachable from the wire. `IcalRecurWeekdayNum::from_str` split a `BYDAY` entry on the last two *bytes*, so `BYDAY=€` cut a character in half and panicked. It now splits on the last two characters. The fuzz setup covered only the syntax tree, which is why this survived, so `fuzz/fuzz_targets/recur.rs` was added: it splits its input into a start and a rule, and asserts decoding never panics, expansion never panics, and a bounded prefix comes out strictly increasing and never before the start.

The second was the scope of a `BYDAY` ordinal. `ordinals` returned `Ignored` whenever `BYMONTHDAY`, `BYYEARDAY` or `BYWEEKNO` was present, so `BYMONTHDAY=15;BYDAY=2MO` meant "the 15th when it is any Monday" instead of "the 15th when it is also the second Monday". RFC 5545 3.3.10 scopes the ordinal by frequency alone, narrowed to the month when `BYMONTH` picks the months of a yearly period, and forbids it outside `MONTHLY` and `YEARLY`; nothing in it voids the ordinal because another part is present, and both oracles count it. This one defect was behind sixteen of the seventeen cases where ical-rs was the lone dissenter.

The third was liveness. `FREQ=SECONDLY;BYSETPOS=2` burned over two minutes of CPU for an answer that is always none: the date-gate skip in `seek` cannot skip a period that `BYSETPOS` empties, though the module documentation claimed otherwise. The old guard, a hundred-year day horizon, also silently truncated a rule that does yield (`FREQ=MONTHLY;INTERVAL=7;BYMONTH=6;BYDAY=SU;BYMONTHDAY=1` stopped at 2183 where both oracles carry on to 2414). Both are now one budget of barren periods per occurrence, shared by `fill` and `seek`, so the work behind any single `next` is bounded whatever shape the rule takes and no satisfiable rule is ever cut short. The whole corpus expands in 75 seconds, worst case about ten milliseconds for a rule that yields nothing.

The parser sweep found no panic and no loss of content across the 191 files: 73 round-trip byte-identically, about 105 differ only where the parser resolves folding, blank lines or QUOTED-PRINTABLE soft breaks, twelve are refused outright as malformed, one is empty. The two observations that came out of it are backlog items, not defects.

Capabilities moved: `recurrence` (the ordinal scope, the liveness bound, and the four settled divergences recorded as current truth).
