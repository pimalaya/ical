---
cairn: delta
change: the-simd-codecs
---

## ADDED Requirements

### Requirement: The CPU token is the application's

The decoders SHALL run the SIMD backend of the token the application installed through simd-cpu (`SimdCpu::install_detected()` in `main`, `SimdCpu::install` without std), and the baseline when none is installed. The crate SHALL NOT detect the CPU itself.

#### Scenario: No token installed
- GIVEN an application that never installs a token
- WHEN a base64 or quoted-printable value is decoded
- THEN it decodes under the baseline backend, to the same bytes a detected token gives

### Requirement: Decoding loses no byte

A decoder SHALL decode every value it is handed rather than fail it, and SHALL drop no byte silently: quoted-printable keeps every byte it cannot read as written, base64 skips a byte outside its alphabet and reports the payload as not well-formed.

#### Scenario: A raw byte in a quoted-printable value
- GIVEN a `DESCRIPTION;ENCODING=QUOTED-PRINTABLE` value holding `caf=E9 \xFF`
- WHEN it is read through `quoted_printable`
- THEN it yields `caf\xE9 \xFF`

#### Scenario: A base64 payload missing its padding
- GIVEN an `ATTACH;ENCODING=BASE64;VALUE=BINARY` value holding `Zm9vYg`
- WHEN it is read through `decode_base64`
- THEN it yields `foob`, the payload reported well-formed

## MODIFIED Requirements

### Requirement: Opt-in content decoding

The core SHALL transform no content. A transfer encoding (`QUOTED-PRINTABLE`, `BASE64`) and a `CHARSET` SHALL be surfaced raw, with their parameters kept. Decoding them is opt-in, one small `no_std` crate per feature: `quoted-printable` through quoted-printable-codec, `base64` through base64-codec, `encoding` through `encoding_rs`. The first two pull simd-cpu through their codec.

#### Scenario: An undecoded binary attachment
- GIVEN an inline `BASE64` value and the `base64` feature off
- WHEN the calendar is decoded
- THEN the value is the raw base64 text, with its `ENCODING` parameter kept

## REMOVED Requirements
