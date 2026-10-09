---
cairn: change
id: a-written-uri-reads-back
status: landed
created: 2026-10-09
---

# A written URI reads back

## Why

rfc5545-contract-fixes writes a URI and a calendar user address verbatim, on the model's encode and through the value cursor. A backslash went out raw, and the decoder, which reads a URI through the text unescape so a `\,` a producer in the wild wrote reads as `,`, dropped it on the way back in: `mailto:a\b@x` written read back as `mailto:ab@x`, and a written value did not round-trip.

## What

Percent-encode a backslash as `%5C` when writing a URI or a `CAL-ADDRESS` value, beside `%0D` and `%0A`, on both write paths. RFC 3986 keeps a raw backslash out of a URI anyway (2.1, 2.2), so `%5C` is the same reference spelled the way the RFC asks. Decoding stays as it is.
