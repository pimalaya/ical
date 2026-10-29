---
cairn: log
change: recur-harness-builds-again
date: 2026-10-29
---

# The recurrence harness builds again

tests/corpus/recur/harness no longer built: its manifest asked ical-rs for a `recur` feature the crate dropped when recurrence became always available. It now asks for no feature. Its own build output (target/, eighteen files) and a stale `__pycache__/dateutil.cpython-314.pyc` from before the oracle was renamed were tracked in the repository; they are deleted, run.sh builds into its work directory instead, Python runs isolated (`-I`), and .gitignore covers both for every corpus harness. The module header named the oracles by their old file names.

A full regeneration in a scratch copy then ran clean: 8,952 cases, the two oracles agreeing on 4,358, ical-rs matching all of them and dissenting on none. The committed consensus.tsv holds 4,331 of those, every one reproduced byte for byte. The 27 others are `FREQ=SECONDLY` rules libical answers in 1.5 to 1.8 seconds against its 2 second alarm, which the run of 2026-08-08 timed out on. The committed file is kept, and tests/corpus/recur/ATTRIBUTION.md now says that the case list, not the answers, depends on the machine.

No capability moved: tooling only.
