# recur

A frozen differential run of the recurrence expander against two independent implementations.

- consensus.tsv: every generated case on which python-dateutil 2.9.0 and libical 3.0.20 answered *alike*, with the answer they agreed on. 4,331 cases, out of 8,952 generated.
- divergence.tsv: the cases where this crate deliberately answers something an oracle does not, each with the reason.

Neither file is anyone else's work: the cases are generated here and the expected answers are what those two libraries computed. The oracles themselves are not vendored, only consulted. python-dateutil is Apache-2.0 or BSD-3-Clause, libical is MPL-2.0 or LGPL-2.1.

## Why two oracles

One oracle is a second opinion; two that agree are evidence. Where dateutil and libical disagree with each other, neither can be called the answer, so the case is dropped rather than arbitrated: 4,621 of the 8,952 went that way, most of them rules RFC 5545 forbids, where dateutil applies the part as a limit and libical refuses the rule.

## Regenerating

harness/run.sh does the whole run and rewrites consensus.tsv. It needs nothing installed: nix fetches an interpreter carrying dateutil and the libical headers. Expect around an hour, most of it the per-case alarm the two oracles need, since a rule neither can satisfy walks to year 9999 one tick at a time.

The harness is four files: src/main.rs (the case generator, the ical-rs runner, and the cross), oracle_dateutil.py, oracle_libical.c, and run.sh. It is a detached cargo package, so it never joins the crate's own build.

divergence.tsv is hand-curated and is not regenerated. When the expander's answer to one of those cases changes, the change is either a bug or a new decision, and both deserve a human.

## Never edit an expected value

A changed answer is a behaviour change. Fix the expander, or record the divergence with its reason; do not make the corpus agree with the code.
