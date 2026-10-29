---
cairn: tasks
change: alarm-instants
---

- [x] Open question on a VTODO trigger without DTSTART settled
- [x] delta.md: a new spec/alarm.md capability, requirements and scenarios for every rule of the proposal
- [x] Module beside recur/, `no_std`, its types named after the crate's prefix
- [x] Relative triggers on start and end, VEVENT and VTODO anchors
- [x] Absolute triggers
- [x] Recurring parents: one firing per occurrence, overrides replacing the series' alarms, `THISANDFUTURE` honoured
- [x] `REPEAT` with `DURATION`
- [x] Floating anchors in a supplied zone; nominal days across a daylight-saving change
- [x] `ACKNOWLEDGED` and an alarm `UID` carried on each firing
- [x] Tests: one per rule, a weekly series with a moved instance, a missing anchor reported
- [x] `cargo fmt`, `cargo test --all-features`, `cargo clippy --all-features`, `cargo build --no-default-features` (no std leak), tarpaulin on the new module
- [ ] Fold into spec/alarm.md, log entry, CHANGELOG under Unreleased

## Notes

Provisional answers from the night session of 2026-10-27, for review:

- **Open question settled provisionally**: a relative trigger counted from the start of a `VTODO` with a `DUE` and no `DTSTART` counts from its `DUE`, as documented leniency (delta.md, "A missing anchor is reported, never guessed"). The recurrence timeline already places such a to-do at its due time, so the leniency costs no code of its own.
- **Not lazy**: `IcalAlarms::firings` returns a sorted `Vec`, not a lazy iterator as the proposal says. Overrides and `RELATED=END` triggers put firings out of the timeline's identity order, so a lazy time-ordered stream would need a buffer anyway; the window bounds the list. The delta says "in time order" and no longer "lazily".
- **A VEVENT on a date with no end** ends the next day, RFC 5545 3.6.1's default the timeline applies, not "the start" as the proposal wrote; a dated event lasts the day.
- **An absolute trigger without its `Z`** (forbidden by RFC 5545) reads on the floating clock: the supplied zone, else UTC.
- **A nominal `DURATION` between repetitions** counts its days on the anchor's wall clock, like the trigger itself; an absolute trigger's repetitions count on UTC.
- **Firing identity**: `alarm` is the index among the `VALARM`s of the component holding it, `over` names that component when it is an override (also for an instance a `THISANDFUTURE` override lends its alarms to). A consumer finds the `VALARM` from the pair.
- `Alarm::fire` carries `#[allow(clippy::too_many_arguments)]` (8 arguments, all needed); a small context struct would remove it if preferred.
- Coverage: tarpaulin on src/alarm.rs with tests/alarm.rs, 95.00% (190/200 lines).
