---
cairn: tasks
change: prop-identity
---

- [x] Add `IcalPropPath::identity` and the rule that computes it
- [x] Pair properties by equality, then identity, then position, refusing a positional pair between differing identities
- [x] Carry the side-measured path on each op, for the source lookup
- [x] Translate a base position through the left side's removals before resolving it
- [x] Resolve a replay target by identity where the path carries one
- [x] Un-ignore the four reproductions
- [x] Teach the property model and the reference merge the same identity
