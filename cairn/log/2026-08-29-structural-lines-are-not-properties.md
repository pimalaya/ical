---
cairn: log
change: structural-lines-are-not-properties
landed: 2026-08-29
---

# A merge never emits a calendar its own parser refuses

`BEGIN` and `END` are read as the component envelope wherever the merge reads a component's properties, so a bare, envelope-less record contributes its real properties alone and no structural keyword is ever spliced into a well-formed calendar. A line the replay copies is given a line ending where the side it came from had none, since the last line of a truncated download would otherwise swallow the line it lands in front of.

The invariant is a law rather than a repair: the generated property suite, the corpus harness over every fixture and the fuzz target's oracle all assert that the merged calendar parses and reparses to its own bytes.

Capabilities moved: merge (ADDED: The merged calendar can always be read back).
