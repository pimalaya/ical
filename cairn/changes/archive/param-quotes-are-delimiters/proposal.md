---
cairn: change
id: param-quotes-are-delimiters
status: landed
created: 2026-08-30
---

# A parameter's double quotes are delimiters, not content

## Why

RFC 5545 section 3.1 gives `param-value = paramtext / quoted-string`, `quoted-string = DQUOTE *QSAFE-CHAR DQUOTE`. The DQUOTE pair is the production's own delimiter: `QSAFE-CHAR` is every character but a control and a double quote, so a quote can never be part of what the pair encloses.

The decoded model keeps that pair today, on purpose (`decoded-model.md`, "A parameter value is encoded by RFC 6868"). That decision is wrong, and it is wrong at the boundary a consumer reads:

- `line.param::<ALTREP>()` on the RFC's own section 3.2.1 example hands back `"cid:part1.0001@example.org"`, quotes included, which no URI parser accepts. `DIR`, `SENT-BY`, `DELEGATED-TO`, `MEMBER` and `SCHEMA` all carry a quoted form in the wild and read back the same way.
- The jCal and JSCalendar exports write that string into JSON, where RFC 7265 and RFC 8984 have no iCalendar quoting. The JSCalendar export already carried a private `unquoted` helper to undo it, whose own documentation said the model "keeps them, since they are bytes the syntax tree has to reproduce". The syntax tree reproduces them from the syntax leaf, not from the model.
- The merge compares parameters through the same decoder, so `PARTSTAT="ACCEPTED"` and `PARTSTAT=ACCEPTED` are two different values, and a calendar whose server re-quotes a parameter reports a change nobody made.
- RFC 6868 `^'` becomes unreachable: a value holding a literal double quote decodes to the same text as a value the wire quoted.

It is also the one place the crate mixes syntax into the decoded model. A value node does not keep its `\,` escapes, a folded line does not keep its folds; both are recorded on the syntax side, which is where the quotes already are.

## What

`unescape_param` strips a balanced surrounding DQUOTE pair before resolving the RFC 6868 carets, and `escape_param` wraps its result in one when the escaped text carries a `,`, a `;` or a `:`, the three delimiters RFC 5545 keeps out of a bare `paramtext`. A fourth, the double quote itself, cannot occur: the caret encoding has already spelled it `^'`.

`Escaper` grows `has_param_quoting`, false for vCalendar 1.0, whose grammar has no `quoted-string` and whose double quote is therefore content. It answers the same way as `has_param_encoding` for both versions this crate knows, but it is a different question about a different RFC, and the two are asked for different reasons.

The JSCalendar export's `unquoted` goes, the codec now doing what it worked around.

Byte fidelity is untouched: the quotes live on the syntax leaf, which parsing and serialization never read through the codec. Only the canonical `decode` and `encode` projections change, and they stay lossless, a value that needs quoting being quoted again.

## Judgement call, for review

**This is a breaking change for a caller that builds parameters by hand.** An `IcalParam::AltRep(Cow::Borrowed("\"cid:...\""))` written with its own quotes now means a value whose text starts and ends with a quote, and goes out as `ALTREP="^'cid:...^'"`. The value is not lost and round-trips, but the calendar is not the one the caller meant, so it lands in the CHANGELOG as a breaking change.
