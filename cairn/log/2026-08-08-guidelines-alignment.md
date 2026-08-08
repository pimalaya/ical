---
cairn: log
change: guidelines-alignment
landed: 2026-08-08
---

# Align the whole repository with the Pimalaya guidelines

A conformance pass over every guideline scope that applies to a library. Most of it was mechanical. Three findings were not.

**crate-003 cost a feature.** "A cargo feature is justified only when it pulls additional crates into the build, std included. When gating some code would not change the crate set at all, do not gate it." The `recur` feature pulled nothing: the module's own documentation said "dependency-free" in the same breath as calling it opt-in. It is gone, and `recur` and `timezone` ship unconditionally. The validation error variant for a bad rule stopped being conditional with it, which removed three `cfg` blocks from a file that reads better without them.

**naming-007 cost a rename.** Every public item carries the domain prefix, with two exceptions neither of which applied, so `Valid<T>` is `IcalValid<T>`. vcard-rs carries the same unprefixed name and will need the same rename; that is its repository's to make.

**inline-004 cost 72 tags.** Bare `//` comments are banned in favour of five tags. Every explanatory comment in the crate is now a `NOTE`, which is what they all were.

The rest, in one list: the manifest matches the template field for field, with alphabetical dependencies that all disable default features, and per-example and per-bench blocks; the README lost its API snippet and gained the library section order, an RFC coverage table naming all ten specs, redirect-only Usage and Examples sections and the standard disclosure, social and sponsoring blocks; the CHANGELOG's Unreleased section is a net diff of the current state rather than a running log, which is what the cairn log beside it is for; imports go through `crate` rather than `super`, one `use` per crate; markdown paths are bare or linked, never backticked; and the Cairn conformance checker is vendored as cairn/verify.sh so the repository needs nothing checked out beside it.

One thing was deliberately left alone. The log entries under cairn/log also carry backticked paths, and Cairn holds that a log entry is immutable once written. History is not restyled, so they stay as they were.

`cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features` (11 binaries), `cargo build --no-default-features` and `cargo deny check` are all green.

Capabilities moved: `recurrence` and `timezone` (MODIFIED: both are available unconditionally, not behind a feature); `conformance` (MODIFIED: the proof marker is named `IcalValid`).
