---
cairn: change
id: a-folded-line-is-stripped-once-assembled
status: draft
created: 2026-11-03
---

# A folded line is stripped once assembled

## Why

Found during the first speed pass ([the-lines-built-in-place](../archive/the-lines-built-in-place/proposal.md)), which kept the behaviour as it was. The tokeniser strips the leading whitespace of a line's first physical line only. A line made entirely of whitespace, a dangling continuation after a blank line, strips to nothing, and the continuation that follows it contributes everything but its one fold marker, leaving the rest in front of the name.

`BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\r\n   \r\n  X-A:b\r\nPRODID:x\r\nEND:VCALENDAR\r\n` parses to a line named ` X-A`. `to_bytes` still gives the input back, the wire shape holding every byte, so the corpus and the parse fuzz target's fixpoint never see it. But decoding the calendar and writing it back puts ` X-A:b` at the start of a line, which the next parse folds into the `VERSION` line before it: the property is lost, and the decoded calendar's text is not a fixpoint.

vcard-rs fixed the same reading on 2026-08-29, found by its merge fuzz target (vcard cairn/log/2026-08-29-a-folded-line-is-stripped-once-assembled.md).

## What

As vcard-rs does (`push_content` in its src/tree/line.rs): a continuation joined to a logical line still empty is stripped of its leading folding whitespace and stray line-break bytes, recorded on the wire shape at the line's start, in both the folded and the `QUOTED-PRINTABLE` join paths, so the line is named `X-A` and still serializes back to its input.

## Consequence

A parsed line's name never begins with whitespace. A calendar holding such a line decodes with the property under its real name, and its text reparses to itself. The bytes `to_bytes` writes do not change.
