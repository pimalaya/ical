---
cairn: change
id: a-write-back-never-breaks-the-line
status: landed
created: 2026-08-29
---

# A write-back never breaks the line

## Why

The merge fuzzer found a third calendar the merge emits and its own parser refuses, `MissingPropertyColon("CATEGORIES;LANGUAGE=en requirements.")`. The right side carried a `CATEGORIES` line whose `LANGUAGE` parameter held a `\n`, and the merged line came out with a real line break in its head, so the first physical line had no colon left.

The replay is at fault, in the one place it does not copy bytes. A parameter action carries the *decoded* parameter and `apply_to_line` writes back `param.encode()`. Decoding a parameter runs its value through the text unescape, which turns `\n` into a newline, and encoding writes the result verbatim, because a parameter value is quoted on the wire rather than backslash-escaped. The pair is therefore not a round trip, and what it puts back can be a byte the head cannot carry.

Probing for the rest of the family found a second instance, reachable from the public API with no merge involved. The vCalendar 1.0 escaper escapes `;` alone, so it writes a newline out raw:

    IcalCst::parse(v1_calendar)?.prop_mut::<SUMMARY>()?.set_text("a\nb")

yields a calendar that no longer parses. The merge reaches it whenever the baseline side is a 1.0 calendar and the replayed item was decoded from a 2.0 one: the item arrives as text holding a newline and is escaped by rules that have nothing to write for it.

Both are the same law. What a write puts into a line has to be bytes that line can carry.

## What

The replay copies a parameter off the source line, as it already copies a whole property, a value and a component. Nothing decoded is written back into a head.

The vCalendar 1.0 escaper writes a newline as `\n`. Versit has no newline escape, so this is the closest 1.0 can carry: the value stays on one line and round-trips from then on, and it reads back as those two characters rather than as a break. The alternative, leaving the raw byte in, is a calendar that cannot be read at all.

## Judgement call, for review

**The parameter decode is left alone.** Unescaping a parameter value is itself questionable, since RFC 5545 section 3.2 gives parameters no backslash escapes and RFC 6868 exists precisely because they have none. Making the decode symmetric with the encode would fix this crash too, and more deeply, but it moves what every decoded parameter says, and with it jCal, JSCalendar and the corpus comparisons. The replay wanting bytes rather than a re-encoding stands on its own, so it is what changes here.
