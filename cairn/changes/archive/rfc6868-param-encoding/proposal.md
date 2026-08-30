---
cairn: change
id: rfc6868-param-encoding
status: landed
created: 2026-08-29
---

# RFC 6868 parameter value encoding

## Why

A parameter value is decoded with the RFC 5545 section 3.3.11 *text* escapes, `\\` `\,` `\;` `\n`. Section 3.2 gives a parameter no backslash escapes at all, which is the whole reason RFC 6868 exists: a parameter that must carry a double quote, a newline or a caret encodes it as `^'`, `^n` or `^^`.

So the crate does the wrong thing twice over. A backslash a parameter legitimately carries, a Windows path in an `ALTREP` or an `X-` parameter, is eaten on the way in and cannot be written back. A real `^n` or `^'` from any RFC 6868 producer is handed to the caller raw, so a `CN` reading `O'Brien, "Bob"` arrives with its encoding showing.

vcard-rs has the identical defect: RFC 6350 section 3.3 requires RFC 6868 for vCard parameters too, and the same text unescaper is pointed at them. The two crates are deliberate twins, so the fix belongs in both or the divergence needs a stated reason.

## What

Decode a parameter value by the RFC 6868 rules and encode it back by them, leaving the text escapes to text values where they belong.

The decoding is: `^n` becomes a newline, `^^` a caret, `^'` a double quote, and a caret before anything else stays a caret with the character after it (RFC 6868 section 3.1, which forbids inventing an error there). Encoding is the inverse, applied only where the parameter is not already quoted around the character.

Postel governs the transition: a value carrying no caret and no backslash means the same under both readings, which is nearly every parameter in the corpus, so the change must be invisible for those. What moves is a value carrying a backslash, which stops being unescaped, and one carrying a caret, which starts being decoded.

Done when a parameter round trips byte for byte through decode and encode, when the three RFC 6868 sequences decode and re-encode, when a lone caret survives untouched, and when the golden corpus is unchanged except where a fixture genuinely carries one of these.

Two things the proposal did not say, settled while it landed. The rules are keyed on the version rather than applied everywhere: RFC 6868 updates RFC 5545 and nothing earlier, so a vCalendar 1.0 caret stays literal, which means a parameter node has to carry an escaper the way a value node does. And the byte-for-byte round trip holds for the canonical spelling, since RFC 6868 decoding is deliberately not injective: `^x` and `^^x` both read as `^x`, and the encoder writes the canonical `^^x` for both.

## Blast radius

The decoded parameter model, jCal and JSCalendar (which carry parameters), the corpus comparisons, and the same list again in vcard-rs for jCard and JSContact. Worth doing before the release rather than after, since it changes what every decoded parameter says.
