---
cairn: tasks
change: the-simd-codecs
---

# Tasks

- [x] Regression tests failing on the old decoders: a raw 8-bit byte among quoted-printable octets, a base64 payload missing its padding.
- [x] Move the `base64` and `quoted-printable` features to base64-codec and quoted-printable-codec 0.1, simd-cpu 0.1 as a dev-dependency, `rust-version` 1.89.
- [x] `IcalBinary::decode_base64` returns the bytes and the well-formed flag; `IcalValueCursor::quoted_printable` keeps every byte.
- [x] tests/codecs.rs: every corpus value decodes under the detected token as under the codecs' `BASELINE`, a planted break in each call site caught.
- [x] The fuzz parse target installs the detected token and runs every decoder.
- [x] The lib.rs header, README, example and CHANGELOG.
- [x] Fold the delta into cairn/spec/decoded-model.md and log.
