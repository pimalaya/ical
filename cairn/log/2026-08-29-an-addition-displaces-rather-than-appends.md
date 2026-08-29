---
cairn: log
change: an-addition-displaces-rather-than-appends
landed: 2026-08-29
---

# The winner of a both-sides-added collision replaces the loser

An addition that wins a collision now replaces the addition it beat, where that one stood, so the merged calendar never holds more members of a group than the side that wrote the most and a position addressing them is not renumbered by the replacement.

Two `LOCATION` lines in one `VEVENT`, which RFC 5545 forbids and this crate's own `validate` refuses, are no longer reachable, and merging a calendar with two byte-identical sides is no longer a mutation.

Capabilities moved: merge (ADDED: An addition that wins replaces the one it beat).
