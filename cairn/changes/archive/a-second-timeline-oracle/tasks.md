---
cairn: tasks
change: a-second-timeline-oracle
---

- [x] Pin an ical.js release and fetch it through nix in a new harness script
- [x] Write its answers in the windows format and cross them with recurring-ical-events
- [x] Rebuild agreement.tsv from series both oracles agree on; add ical.js's answer to every divergence
- [x] Vendor ical.js and libical test calendars where licences allow, with attribution
- [ ] Export calendars from the Google, Microsoft and Fastmail test accounts (read-only), scrub them, add them (moved to timeline-invariants-and-live-calendars)
- [ ] Proptest invariants: order, dedup, overlap, window equals slice, bounded work (moved to timeline-invariants-and-live-calendars)
- [x] Update the timeline corpus requirement in spec/recurrence.md (two oracles), log
