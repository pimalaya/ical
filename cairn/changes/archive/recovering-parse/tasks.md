---
cairn: tasks
change: recovering-parse
---

- [x] Add an opaque item to the syntax tree for a line that cannot be structured
- [x] Add the recovering parse entry point, leaving the strict one the default
- [x] Recover from a missing END by closing the component at end of input
- [x] Report the recovered lines to the caller
- [x] Prove the twelve refused fixtures parse, round-trip and report
