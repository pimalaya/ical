# timeline

A frozen differential run of the timeline (`IcalRecurTimeline`, occurrences as instants over a window) against recurring-ical-events 3.9.0, over the calendars of tests/corpus/rie.

- windows.tsv: the window each calendar is walked over, from a year before its earliest `DTSTART`, `DUE` or `RECURRENCE-ID` to three years after its latest.
- agreement.tsv: every series on which ical-rs and the oracle answered alike, with that answer: the earliest 200 occurrences starting in the window, start and end each. 966 series.
- divergence.tsv: the series where this crate deliberately answers something the oracle does not, each with the reason. 12 series.

Neither file is anyone else's work: the windows are chosen here and the answers are what the two implementations computed. The oracle is not vendored, only consulted, and its calendars are vendored in tests/corpus/rie under their own licence.

## One oracle

The recurrence corpus crosses two oracles and keeps only what they agree on. Here there is one, since recurring-ical-events is the one independent implementation of this layer with a real-world suite behind it. A series both answer alike is kept as it comes, a guard against regression rather than a proof. Every series the two answered differently was read by hand on 2026-10-29 and became either a fix to this crate (landed as what-the-timeline-oracle-found and timeline-oracle-second-round) or a line of divergence.tsv.

## Regenerating

harness/run.sh does the whole run and rewrites windows.tsv and agreement.tsv. It needs nothing installed: nix builds an interpreter carrying recurring-ical-events from nixpkgs. The series that differ go to the work directory's disagreement.txt, which must hold exactly the series of divergence.tsv, plus the calendar the oracle refuses outright (bad_rrule_missing_until_event.ics).

The harness is four files: src/main.rs (the windows, the ical-rs runner, and the cross), oracle_rie.py, run.sh, and header.txt. It is a detached cargo package, so it never joins the crate's own build, and run.sh builds it into the work directory.

## Never edit an expected value

A changed answer is a behaviour change. Fix the crate, or record the divergence with its reason; do not make the corpus agree with the code.
