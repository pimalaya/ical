---
cairn: log
change: the-simd-codecs
landed: 2026-11-03
---

# The SIMD codecs

Landed base64 and quoted-printable over the Pimalaya SIMD codecs, in place of the base64 0.23 and quoted_printable 0.5 crates, as vcard-rs and mime-rs did on 2026-11-02. Both decode under `SimdCpu::current()`, the token the application installed through simd-cpu or the baseline; the crate never detects the CPU. base64-codec 0.1, quoted-printable-codec 0.1 and simd-cpu 0.1 come from crates.io, simd-cpu as a dev-dependency only (tests, the content_encodings example and the fuzz crate install the detected token). `rust-version` is 1.89.

`IcalBinary::decode_base64` now returns `Option<(Vec<u8>, bool)>`, the bytes and whether the payload was well-formed, and decodes liberally: line breaks and blanks skipped, other bytes outside the alphabet skipped and reported, missing padding accepted. `Zm9vYg` used to fail. `IcalValueCursor::quoted_printable` keeps its `Vec<u8>` and loses the raw fallback, the codec never failing; it used to drop every byte outside printable ASCII and the blanks before it (`caf=E9 \xFF` read `caf\xE9`). Both cases have a test in tests/encoding.rs that failed on the old decoders.

tests/codecs.rs installs the detected token and checks every quoted-printable and base64 value of the corpus (5 and 2 on 2026-11-03) decodes as the codecs' `BASELINE` does; a break planted in each call site was caught. The fuzz parse target installs the detected token, decodes every binary, quoted-printable and charset value, and checks a quoted-printable value never grows.

Verification: the suite under default, all and no features; clippy `--all-targets --all-features -D warnings`; `cargo build --no-default-features` depends on nothing, `--features base64,quoted-printable` on the two codecs and simd-cpu; the parse fuzz target for 180 seconds (116,011 runs, coverage 6,018), no crash. Not run: the thumbv6m-none-eabi and riscv32imc-unknown-none-elf builds, the devshell carrying neither target. Seen and left alone, both older than this change: clippy `--no-default-features` fails on five unused-variable and unused-mut lints in src/recur/scale.rs, and rustdoc with `-Dwarnings` on two bare URLs in src/tzdb/windows.rs.

Capabilities moved: decoded-model (ADDED: the CPU token is the application's, decoding loses no byte; MODIFIED: opt-in content decoding names its crates).
