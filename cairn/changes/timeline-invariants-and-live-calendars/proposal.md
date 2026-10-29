---
cairn: change
id: timeline-invariants-and-live-calendars
status: active
created: 2026-10-29
---

# Timeline invariants and live calendars

## Why

Left over from [a-second-timeline-oracle](../archive/a-second-timeline-oracle/proposal.md): the corpus holds only test-suite calendars, and no invariant is checked beyond what an oracle answers.

## What

1. Proptest invariants over generated sets and zones: occurrences sorted and deduplicated, each overlapping the window, a window equal to the slice of a longer walk, bounded work per `next`.
2. Calendars exported read-only from the Google, Microsoft and Fastmail test accounts (never Posteo), scrubbed, added to the timeline corpus.
