---
cairn: change
id: corpus-parity
status: landed
created: 2026-08-08
---

# Test surface to vcard-rs parity

## Why

vcard-rs has 187 test functions, 146 real-world fixtures across eight vendor directories, and competing crates as dev-dependencies for differential tests. ical-rs has 93 tests and six fixtures, while 191 real `.ics` files have already been swept through its parser and are sitting in nobody's repository.

## What

Import those 191 files from the libical, ical4j and ical.js test suites (all permissively licensed, the same move vcard-rs made with calcard, ez-vcard and sabre), with an ATTRIBUTION.md per source directory, and add `calcard` as a dev-dependency for a cross-implementation decode comparison the way vcard-rs's tests/calcard.rs does for cards.

The harness must classify rather than assert byte-identity: of the 191, only 73 are byte-identical today, for the reason the fold-preservation change addresses. Assert instead that every fixture parses, that its output is a serialize fixpoint, that decoding never panics, and that decode, encode and decode again is stable.

Done when the corpus is committed with attribution and the classification is asserted per directory with an exact fixture count.
