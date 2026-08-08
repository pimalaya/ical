---
cairn: tasks
change: fold-preservation
---

- [x] Record the fold points, the folding whitespace and the leading blank lines on the parsed line
- [x] Re-emit them from the byte serializer
- [x] Drop the recorded shape when the line's value is edited, so output stays valid
- [x] Keep the QUOTED-PRINTABLE soft-break join round-tripping
- [x] Prove the fixpoint still holds on re-emitted folded output
- [x] Align the README and the src/lib.rs header with the new guarantee
