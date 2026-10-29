---
cairn: change
id: a-second-timeline-oracle
status: landed
created: 2026-10-29
---

# A second timeline oracle

## Why

Rule expansion is crossed against two oracles, so an answer two independent implementations agree on is evidence. The timeline (sets, overrides, zones, ends as instants) has one: recurring-ical-events, over its own 95 test calendars. One oracle is a second opinion, and its test calendars are the only real-world input so far.

## What

1. ical.js (MPL-2.0, the engine of Thunderbird's and Nextcloud's web calendars) as the second oracle: it reads a calendar's own `VTIMEZONE`s and expands sets with overrides itself, so it shares no code with recurring-ical-events or with libical. A harness beside tests/corpus/timeline/harness, fetched through nix and pinned to one release, writing the same windows format.
2. Cross the two oracles like the recur corpus does: a series both agree on enters agreement.tsv, one they split on is dropped from it, and every line of divergence.tsv gets ical.js's answer beside it. A divergence ical.js agrees with us on is worth saying in its reason.
3. More real-world input: the vendored test calendars of ical.js and libical, where their licences allow vendoring (attribution as tests/corpus/rie does), and exports from the Google, Microsoft and Fastmail test accounts, never Posteo.
4. Invariants by proptest (already a dev-dependency) over generated sets and zones, which no oracle needed: occurrences sorted and deduplicated, every one overlapping the window, `expand` over a window equal to the slice of a longer walk, and bounded work per `next`.

## Consequence

agreement.tsv may shrink: cases recurring-ical-events alone answered and ical.js answers otherwise leave it, which is the point. The replay test keeps its shape; its counts change.
