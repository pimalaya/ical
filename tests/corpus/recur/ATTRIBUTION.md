# recur

A frozen differential run of the recurrence expander against two independent implementations.

- consensus.tsv: every generated case on which python-dateutil 2.9.0 and libical 3.0.20 answered *alike*, with the answer they agreed on. 4,331 cases, out of 8,952 generated.
- divergence.tsv: the cases where this crate deliberately answers something an oracle does not, each with the reason.
- skip.tsv: the RFC 7529 `SKIP` cases, against libical alone.
- rscale.tsv: the RFC 7529 `RSCALE` cases, ical-rs built with its `rscale` feature against libical 3.0.20 built with ICU 76.1, every case the two answered alike. 1,912 cases, out of 2,548 generated (14 scales, 26 rule shapes, 7 starts); libical refused or gave up on 178.
- rscale-divergence.tsv: one case per class of the 458 the two answered differently, with ical-rs's answer and the reason. Every class but one is libical parting from RFC 7529 or from a third implementation; past 2100 the Chinese calendar is approximated differently by both libraries and neither is known right.

Neither file is anyone else's work: the cases are generated here and the expected answers are what those two libraries computed. The oracles themselves are not vendored, only consulted. python-dateutil is Apache-2.0 or BSD-3-Clause, libical is MPL-2.0 or LGPL-2.1.

## Why two oracles

One oracle is a second opinion; two that agree are evidence. Where dateutil and libical disagree with each other, neither can be called the answer, so the case is dropped rather than arbitrated: 4,621 of the 8,952 went that way, most of them rules RFC 5545 forbids, where dateutil applies the part as a limit and libical refuses the rule.

## Regenerating

harness/run.sh does the whole run and rewrites consensus.tsv. It needs nothing installed: nix fetches an interpreter carrying dateutil and the libical headers. Expect around an hour, most of it the per-case alarm the two oracles need, since a rule neither can satisfy walks to year 9999 one tick at a time.

That alarm makes the case list depend on the machine, not the answers. Some `FREQ=SECONDLY` cases take libical 1.5 to 1.8 seconds against its 2 second budget, so a slower or busier run drops them as timeouts. A run on 2026-10-29 reproduced all 4,331 committed cases byte for byte and agreed on 27 more such cases, ical-rs matching every one. The committed file was kept as it is.

The harness is src/main.rs (the case generators, the ical-rs runner, and the cross), oracle_dateutil.py, oracle_libical.c, run.sh and rscale.sh, with the two corpus headers. It is a detached cargo package, so it never joins the crate's own build, and run.sh builds it into the work directory.

harness/rscale.sh regenerates rscale.tsv the same way, in a minute, and leaves the differing cases in the work directory's disagreement.tsv. One oracle rather than two, because only one exists: python-dateutil does not parse `RSCALE`. libical reaches another scale through ICU4C and ical-rs through ICU4X, so the two share the CLDR algorithms and no code.

divergence.tsv is hand-curated and is not regenerated. When the expander's answer to one of those cases changes, the change is either a bug or a new decision, and both deserve a human.

## Never edit an expected value

A changed answer is a behaviour change. Fix the expander, or record the divergence with its reason; do not make the corpus agree with the code.
