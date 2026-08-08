---
cairn: log
change: test-surface
landed: 2026-08-08
---

# Close the test-surface gap against vcard-rs

Three test files, and three defects they found. Coverage went 78.08% to 80.75%, which is the less interesting half of the result.

## What was untested

**The content-decoding features.** `quoted-printable`, `base64` and `encoding` are on by default, they are the only code in the crate that touches raw bytes with intent, and they had two unit tests between them. tests/encoding.rs is fifteen, and it tests the core's side of the bargain first: a `QUOTED-PRINTABLE` value, a base64 payload and a value in a foreign charset all reach a caller as the bytes the wire carried, with the parameters that say how to read them still attached. Only then does it test that each helper decodes what it claims. The stacking order is asserted too, since `CHARSET=ISO-8859-1` with `ENCODING=QUOTED-PRINTABLE` only reads correctly one way round.

**The vocabularies.** Four closed enums, each with a `Deref` arm and a `FromStr` arm per variant, and nothing walking them. tests/coverage.rs walks all four whole, checks that no two variants share a wire name, drives every value kind through a calendar and back, and asserts the maximal calendar is byte-faithful. `IcalValueKind::ALL` and `IcalParamKind::ALL` were added to make the walk a loop, which the other two vocabularies already had.

**The validators.** tests/validate.rs reaches every variant of both error enums, asserts that each reports every problem rather than the first, and covers the proof marker: what goes into `IcalValid` comes out, and it converts back to a syntax tree.

## What the tests found

**The version axis was empty.** Not one of the seventy property lenses overrode `allowed_versions`, so every property claimed to exist in both vCalendar 1.0 and iCalendar 2.0, and `IcalValidateError::PropVersion` could never fire. The CHANGELOG advertised it. This is the cardinality axis all over again, found the same way and filled the same way: forty-four properties now name their versions, six of them vCalendar 1.0 alone (the four legacy alarm properties, `RNUM` and `TZ`) and thirty-eight iCalendar 2.0 alone. The other twenty-six genuinely predate the split.

**The spec dispatch could not check itself.** Seventy hand-written arms mapping a kind to a marker, with nothing asserting that a marker under an arm answers for that arm's property. Both vtables now carry the kind their marker declares, and a unit test walks the dispatch against it. Two further invariants come free: every property allows at least one value kind, and the kind in force with nothing declared is one of them.

**A parse error was doing validation's job.** `IcalRecurRule::parse` refused a rule carrying both `UNTIL` and `COUNT`, which made the whole calendar unreadable over a constraint RFC 5545 states about meaning, not syntax. The crate's own Postel's law says strictness lives on the way out, and the validator already had `UntilWithCount` waiting for it. The parse-time check is gone, the error variant with it.

## What was deliberately left

Roughly sixty property lenses sit at 33% coverage, and all of it is one uncalled `cursor()` per file. Writing sixty lines that name sixty markers would move the number by two and a half points and prove nothing that is not already proven by the codec sweep. The guideline says to aim for the number and never twist for it; this is the twist.

`cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features` (17 binaries) and `cargo deny check` are green.

Capabilities moved: `decoded-model` (ADDED: every property belongs to a version); `conformance` (ADDED: the spec dispatch answers for the property it is asked about); `recurrence` (MODIFIED: liberal rule parsing now covers `UNTIL` with `COUNT`).
