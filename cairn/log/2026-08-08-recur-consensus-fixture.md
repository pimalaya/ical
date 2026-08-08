---
cairn: log
change: recur-consensus-fixture
landed: 2026-08-08
---

# Freeze the recurrence consensus as a committed fixture

The differential run of 2026-08-08 was the crate's strongest evidence and it lived in a scratch directory. It is now two files in the repository and a test that replays them, and the harness that produced them is committed beside them rather than described in prose.

The harness is four pieces under `tests/corpus/recur/harness`: a detached cargo package that generates the cases, answers them through ical-rs and crosses the three answer files, plus `oracle_dateutil.py` and `oracle_libical.c`, each with its own per-case alarm, and a `run.sh` that fetches everything from nix so nothing has to be installed. Two things cost an hour to rediscover and are now written down in it: `nix shell nixpkgs#python3Packages.python-dateutil` puts the library on the store but on no interpreter's path, so the script builds an interpreter that carries it; and libical's headers live in its `dev` output, which `nix shell` puts on no search path either.

The run: 8,952 generated cases, every frequency crossed with every `BY` part, twelve interacting pairs, the `UNTIL` and `COUNT` bounds, twelve composite rules of the shape real calendars carry, and six thousand seeded random combinations of two to four parts, each from one of eight starts chosen for their edges. python-dateutil 2.9.0 and libical 3.0.20 answered every one.

**The two oracles agree on 4,331 of them. ical-rs matches all 4,331.** That is the file `consensus.tsv`, and `tests/recur_corpus.rs` replays it in three seconds with no Python and no C toolchain in sight.

The 4,621 cases the oracles split on are dropped rather than arbitrated, and they are not noise: most are rules RFC 5545 forbids, where dateutil applies the part as a limit and libical refuses the rule outright. Neither can be called the answer, so neither is.

Freezing it corrected the record. Two of the four divergences the plan called settled turned out not to be divergences at all:

- **`BYWEEKNO` and the ISO week-year.** The plan said python-dateutil numbers weeks within the calendar year and we part from it. It agrees with us on every comparable case in the corpus. The real divergence is from libical, on 70 cases.
- **`BYSETPOS` on the whole period.** The plan said dateutil truncates the first period at `DTSTART` first, and that we part from it. It does not: `DTSTART=20260115T090000` with `FREQ=MONTHLY;BYDAY=MO;BYSETPOS=1` gives February's first Monday from both, because January's is selected and then dropped. The reading is still worth recording, since the RFC leaves it open and the two orders give different answers, but it is not a disagreement with anybody.

The other two are real and measured: `BYSETPOS` at `DAILY`, `WEEKLY` and `HOURLY`, which libical ignores (239 cases), and a part its frequency forbids, which we ignore whole where dateutil limits and libical refuses. All four are in `divergence.tsv` with the reason, and the four settled requirements now say what the corpus shows rather than what the earlier run's notes remembered.

Capabilities moved: `recurrence` (ADDED: the differential corpus is replayed; MODIFIED: the four settled behaviours, restated against the measured evidence).
