---
cairn: log
change: a-written-uri-reads-back
date: 2026-10-09
---

# A written URI reads back

A URI and a calendar user address are written verbatim since rfc5545-contract-fixes, and the decoder reads both through the text unescape, which is what lets a `\,` from a producer in the wild read as `,`. A backslash written raw was therefore lost on the next read: the value did not round-trip.

`verbatim_node` (tree/codec/encode.rs), which the model's encode and the value cursor's `set_text` and `set_bytes` both go through, now writes `\` as `%5C`, beside `%0D` and `%0A`. RFC 3986 keeps a raw backslash out of a URI, so `%5C` is the same reference in the spelling the RFC asks for. A value read back is the `%5C` form, which writes back the same bytes.

Capabilities moved: decoded-model, parsing.
