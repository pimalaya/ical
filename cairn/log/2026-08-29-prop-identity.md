---
cairn: log
change: prop-identity
landed: 2026-08-29
---

# Address a property by what it names, not by where it sits

`IcalPropPath` gained an `identity`: the value of a property that may occur more than once and whose value names a thing outside the calendar, which RFC 5545 and RFC 7986 give to `ATTENDEE`, `ATTACH`, `RELATED-TO`, `CONFERENCE` and `IMAGE`. The diff pairs same-named properties by equality, then by identity, and only then by position, and it refuses a positional pair between two identities that differ: a different calendar address is a different person. The replay resolves its target by identity where the path carries one, reads its new bytes from the position the line holds in the side that wrote it, and translates a base position through the baseline side's own removals before resolving it against the merged calendar.

An attendee's answer can no longer be recorded against another attendee, a merge against an untouched side now returns the other side byte for byte, and a removal beside a contested neighbour neither swallows the collision nor duplicates the neighbour.

Capabilities moved: merge (ADDED: Property identity; MODIFIED: Instance identity, A side's own actions all land).
