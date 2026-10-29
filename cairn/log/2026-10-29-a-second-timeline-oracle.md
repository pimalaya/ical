---
cairn: log
change: a-second-timeline-oracle
date: 2026-10-29
---

# A second timeline oracle

- ical.js 2.2.1 is the second timeline oracle (harness/oracle_icaljs.mjs), with @nextcloud/timezones 1.0.2 for undefined `TZID`s, as its consumers do.
- The harness also walks the vendored ical.js and libical calendars (181 in all): agreement.tsv 924 series, new generated split.tsv 82 (oracles split, one sides with us), divergence.tsv 7 (was 12). No series has both oracles against us.
- Left open in timeline-invariants-and-live-calendars: proptest invariants and test-account exports.

Capabilities moved: **recurrence**.
