---
cairn: log
change: a-write-back-never-breaks-the-line
landed: 2026-08-29
---

# A write-back never breaks the line

The merge fuzzer found a third calendar the merge emits and its own parser refuses: `MissingPropertyColon("CATEGORIES;LANGUAGE=en requirements.")`. The right side's `CATEGORIES` carried a `LANGUAGE` parameter whose value held a `\n`, and the merged line came out with a real line break in its head, so its first physical line had no colon and everything after the break was orphaned.

The replay was at fault, in the one place it did not copy bytes. A parameter action carries the decoded parameter, and `apply_to_line` wrote back `param.encode()`. That pair is not a round trip: decoding a parameter runs its value through the text unescape, which resolves `\n` to a newline, while encoding writes the result verbatim, because a parameter value is quoted on the wire rather than backslash-escaped. QUOTED-PRINTABLE was in the artifact but not in the fault: the encoding parameter was only another parameter the right side dropped. The replay now copies the parameter off the source line, as it already copies a whole property, a value and a component.

Probing the rest of the family found a second instance, reachable from the public API with no merge involved. The vCalendar 1.0 escaper escaped `;` alone, so `set_text("a\nb")` on a 1.0 calendar wrote the newline out raw and the calendar stopped parsing. The merge reached it whenever the baseline side was a 1.0 calendar and the replayed list item had been decoded from a 2.0 one. A newline is now written as `\n`, which versit has no escape for and which therefore reads back as those two characters. That is the closest 1.0 can carry it, and it is the one place the escaper and the unescaper are not exact inverses, which the codec now says.

The fuzzer's calendar is kept as a merge test in its reduced shape, beside the 1.0 list-item case and a cursor test for the public path. All three merge artifacts were rechecked and none reproduces, so the directory is empty again.

Spec updated: `merge` (MODIFIED: the merged calendar can always be read back), `parsing` (ADDED: a written value never breaks its line).
