---
cairn: log
change: cairn-adoption
landed: 2026-08-08
---

# Adopt Cairn, retiring docs/

`docs/` held a README index and one living `plan.md` carrying three different kinds of writing at once: what had landed, what was settled, and what was still to do. Cairn splits those three onto their own axes, so this repository now keeps `cairn/spec/` (current truth, one file per capability), `cairn/changes/` (in-flight proposals) and `cairn/log/` (dated history), with `cairn.toml` as the root marker and `AGENTS.md` (plus `CLAUDE.md`) as the activation stanza.

The migration was mechanical. The "Landed" section of `docs/plan.md` became the log entry beside this one. The "Settled" section became requirements in `cairn/spec/recurrence.md`, since a deliberate divergence from another implementation is current truth, not history. The architecture summarised in the `src/lib.rs` header was seeded into `cairn/spec/parsing.md`, `cairn/spec/decoded-model.md` and `cairn/spec/conformance.md`, which is a backfill Cairn normally discourages, done once here because the behaviour it describes already exists and the backlog below needs something to state its deltas against. Each of the twelve backlog items became a change folder under `cairn/changes/`. `docs/` was deleted.

The `src/lib.rs` header remains the entry point for the code. The spec is the behavioural truth behind it, and the forcing rule now applies: a behaviour change is not done until its delta is folded into the spec and an entry is appended here.

Capabilities moved: none. This change moved documentation, not behaviour.
