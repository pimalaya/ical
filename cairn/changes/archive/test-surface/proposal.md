---
cairn: change
id: test-surface
status: landed
created: 2026-08-08
---

# Close the test-surface gap against vcard-rs

## Why

Measured against the sibling crate, ical-rs was the broader library and the less evenly covered one: 78.08% against 83.16%, with three holes vcard-rs does not have. The three optional content-decoding features, all on by default, had no integration test at all. There was no sweep over the closed vocabularies, so a `Deref` arm spelling a name differently from the `FromStr` arm that parses it had nowhere to fail. And the two validators, the only place the crate ever says no, were tested only through the unit tests of the module they live in.

## What

One test file per hole, and whatever the tests turn out to expose.
