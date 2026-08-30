---
cairn: log
change: param-quotes-are-delimiters
date: 2026-08-30
---

# A parameter's double quotes are the grammar's, not the value's

The decoded model held a parameter exactly as the wire spelled it, its RFC 5545 section 3.1 quotes included. Every consumer read them: `line.param::<ALTREP>()` on the RFC's own section 3.2.1 example handed back `"cid:part1.0001@example.org"`, quotes and all, which no URI parser takes; the jCal export wrote them into a JSON string that has no iCalendar quoting; and the merge, comparing through the same decoder, read `PARTSTAT="ACCEPTED"` and `PARTSTAT=ACCEPTED` as two values, so a server that re-quoted a parameter reported a change nobody made.

The JSCalendar export had already noticed, and worked around it with a private `unquoted` helper whose own documentation stated the model "keeps them, since they are bytes the syntax tree has to reproduce". The tree reproduces them from the syntax leaf; the model never needed them. That helper is gone.

`unescape_param` now strips a balanced surrounding pair before resolving the carets, and `escape_param` wraps its result in one when the text carries a `,`, a `;` or a `:`. `Escaper::has_param_quoting` keeps vCalendar 1.0 out of it, its grammar having no `quoted-string`, and RFC 6868 `^'` becomes reachable: a literal double quote is content and encodes as itself.

Byte fidelity is untouched, the quotes living on the syntax leaf that parsing and serialization never read through the codec. The canonical `decode` and `encode` projections change and stay lossless: a value that needs quoting is quoted again, and `PARTSTAT="ACCEPTED"` comes back as `PARTSTAT=ACCEPTED`, the quotes having had nothing to protect.

It breaks a caller that built a parameter with its own quotes: `IcalParam::AltRep("\"cid:...\"")` now means a value whose text starts and ends with a quote and goes out as `ALTREP="^'cid:...^'"`. Nothing is lost, but the calendar is not the one that caller meant.

vcard-rs carried the identical defect and is fixed in the same breath, the two crates being deliberate twins. Its `has_param_quoting` names three versions where this one names two, vCard 3.0 having `quoted-string` without RFC 6868's caret encoding, which iCalendar has no equivalent of.

Capabilities moved: decoded-model.
