---
cairn: change
id: a-chrono-conversion
status: active
created: 2026-08-30
---

# A chrono conversion

## Why

The sibling of [a-jiff-conversion](../a-jiff-conversion/proposal.md), for the other date library a Rust caller is likely to already hold. Same reason: an occurrence is a civil date and time, everyone else has a type for one, and the conversion should not be written twice by every caller.

There is a second reason here that jiff does not have. `recurrence`, the crate this one is measured against, expands in `chrono::DateTime<Tz>` and therefore crosses a partial function on every step, which loses occurrences in zones whose transition falls at midnight. `chrono::NaiveDateTime` was in the same crate the whole time. Converting to the naive type and refusing to convert to the zoned one says which of the two is the right target more clearly than prose can.

## What

An off-by-default `chrono` feature, mirroring the jiff one exactly:

- `TryFrom<IcalRecurDateTime> for chrono::NaiveDateTime`
- `From<chrono::NaiveDateTime> for IcalRecurDateTime`

Outward is fallible for the same reason: this crate admits second 60 and chrono represents a leap second through a nanosecond overflow rather than a second field, so a mapping would be a decision. It is refused instead. chrono's year range is wide enough that it is not the constraint jiff's is.

**No `DateTime<Tz>` conversion**, for the reason the jiff change gives, and here with a worked example behind it.

## Consequence

The same as its sibling: an optional dependency in the model layer, and a `no_std` property to verify rather than assume. chrono 0.4 has been stable for long enough that version churn is not the concern it is for jiff.
