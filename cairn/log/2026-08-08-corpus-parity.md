---
cairn: log
change: corpus-parity
landed: 2026-08-08
---

# Test surface to vcard-rs parity

190 real-world fixtures now live in the repository, taken from the three suites the parser sweep of 2026-08-08 already ran against: libical (40, from `test-data/`, `src/test/` and `src/Net-ICal-Libical/test-data/`), ical4j (104, its `valid`, `invalid` and `hcalendar` samples) and ical.js (46, its samples, its bundled time zones and its parser fixtures). Each directory carries an `ATTRIBUTION.md` naming its source, its licence and why those fixtures are worth keeping. libical and ical.js are Mozilla Public License 2.0, ical4j is BSD 3-Clause; the fixtures keep their own licence, which the attribution says plainly rather than calling all three "permissive" as the backlog item did.

The harness classifies rather than asserting byte-identity, since only 72 of the 190 come back byte-identical today (the reason is the fold-preservation change, next in the backlog). Every fixture, whatever its outcome, must parse, serialize to a fixpoint, decode without panicking, and survive a decode, encode and decode again unchanged. What varies is the outcome, and the outcome counts are asserted per directory, so a fixture that moves between outcomes has to be read as the behaviour change it is:

| corpus  | identical | normalised | refused | empty |
| ------- | --------- | ---------- | ------- | ----- |
| libical | 7         | 23         | 9       | 1     |
| ical4j  | 38        | 64         | 2       | 0     |
| icaljs  | 27        | 19         | 0       | 0     |

The fixtures are read as bytes now, not as text: four libical fixtures and one ical4j fixture are not valid UTF-8, and the old harness would have panicked on them rather than proving they survive.

calcard 0.3 came in as a dev-dependency for a real cross-implementation comparison, not just a corpus sweep: both crates read the same fixture and state the shape they read, which is how many times each `COMPONENT/PROPERTY` pair occurs. `VERSION` is dropped from both shapes, since ical-rs lifts it into a typed indicator and calcard leaves it in the property list, which is a modelling choice rather than a reading of the wire. The two agree on 6 of 6 RFC fixtures, 46 of 46 ical.js, 100 of 104 ical4j (1 differing shape, 2 that only calcard parses, 1 not UTF-8) and 25 of 40 libical (4 differing, 1 only ours, 4 only theirs, 2 neither, 4 not UTF-8). The six fixtures calcard parses and ical-rs refuses are the recovering-parse change's target list.

`cargo test --all-features` runs 103 tests, up from 93.

Capabilities moved: `parsing` (ADDED: the real-world corpus is swept).
