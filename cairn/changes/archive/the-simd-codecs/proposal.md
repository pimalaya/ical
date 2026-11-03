---
cairn: change
id: the-simd-codecs
status: landed
created: 2026-11-03
---

# The SIMD codecs

## Why

mime-rs and vcard-rs decode base64 and quoted-printable through the Pimalaya SIMD codecs since 2026-11-02 (vcard cairn/log/2026-11-02-the-simd-codecs.md), both reading the CPU token the application installs once through simd-cpu. ical-rs still ran the base64 0.23 and quoted_printable 0.5 crates, scalar, each with a rule at odds with the crate's liberal parsing: base64 failed a payload missing its padding or holding a stray byte, and quoted_printable's robust mode dropped every byte outside printable ASCII and the blanks before it, `caf=E9 \xFF` reading `caf\xE9`. Item 9 of the-structure-scanned-once left this as a change of its own.

simd-cpu, base64-codec and quoted-printable-codec are on crates.io at 0.1.0 since 2026-11-03, so ical-rs takes them from the registry, with no git pin to keep in step with its twins.

## What

- The `base64` feature runs on base64-codec, `quoted-printable` on quoted-printable-codec, both through their `default()`, which reads `SimdCpu::current()`. simd-cpu is a dev-dependency only: tests, the content_encodings example and the fuzz crate install the detected token. `rust-version` moves to 1.89, the codecs'.
- **Breaking**: `IcalBinary::decode_base64` returns `Option<(Vec<u8>, bool)>`, the bytes and whether the payload was well-formed, as `VcardBinary::decode_base64` does.
- `IcalValueCursor::quoted_printable` keeps its `Vec<u8>` and drops its raw fallback, the codec never failing.
