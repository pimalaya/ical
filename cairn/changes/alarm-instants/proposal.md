---
cairn: change
id: alarm-instants
status: active
created: 2026-10-27
---

# When an alarm fires

## Why

The crate models VALARM (RFC 5545 3.6.6), `TRIGGER`, `REPEAT`, `DURATION`, `ACTION` and RFC 9074's `ACKNOWLEDGED`, and expands recurrence sets, but answers nothing about when an alarm fires. Three consumers need that answer and must agree on it: calendula's and unfog's `--alarm` (to show and check what a flag wrote), and the Pimalaya alarm daemon (to fire them). The Android app computes a lead time of its own in android/rust/src/calendar/phone.rs to mirror reminders to the phone. Resolving a trigger is RFC 5545 semantics, so it belongs here, once, I/O-free.

## What

A pure resolution, `no_std` with `alloc`, beside recurrence:

- **Input**: a parent component (VEVENT or VTODO) and its overrides, the zones the calendar defines, and a window of instants.
- **Output**: every firing inside the window, lazily, in time order, each naming the alarm (its index, and its `UID` when RFC 9074 gives one), the occurrence it belongs to (the recurrence id, absent for a single item) and the instant.

Rules, each from RFC 5545 3.8.6.3 unless named:

- A `DURATION` trigger counts from the parent's start, or from its end with `RELATED=END`.
- A VEVENT's start is `DTSTART`; its end is `DTEND`, else `DTSTART` plus `DURATION`, else the start (3.6.1).
- A VTODO's start is `DTSTART`; its end is `DUE`, else `DTSTART` plus `DURATION`.
- An absolute trigger (`VALUE=DATE-TIME`) is a UTC instant and fires once, whatever the recurrence.
- A relative trigger fires once per occurrence, counted from that occurrence's start or end; an override's own VALARMs replace the series' for its instance.
- `REPEAT` and `DURATION` together add that many firings, each `DURATION` after the previous; one without the other is invalid and adds none.
- A trigger whose anchor is missing (a relative trigger on a VTODO with neither `DTSTART` nor `DUE`) resolves to nothing and is reported as such, never guessed.
- A floating anchor resolves in a zone the caller supplies, as recurrence instants do (spec/tz.md).
- `ACKNOWLEDGED` (RFC 9074 6) is carried on each firing so a consumer can skip what was dismissed before it; the resolution itself filters nothing.

Nominal and exact durations follow 3.3.6: weeks and days are nominal (a day across a daylight-saving change stays a calendar day), hours, minutes and seconds exact.

## Out

- Firing, snoozing, writing `ACKNOWLEDGED`: the consumer's.
- RFC 9074 `PROXIMITY` triggers: they need a location, not a time.
- The vCalendar 1.0 `AALARM`, `DALARM`, `MALARM`, `PALARM` properties.

## Open questions

- **A relative trigger on a VTODO without DTSTART.** Some clients write `TRIGGER:-PT15M` on a to-do carrying only `DUE`, meaning "before due". RFC 5545 makes it invalid. Resolve it against `DUE` as a documented leniency, or refuse it.
