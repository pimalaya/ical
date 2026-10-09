---
cairn: log
change: a-raw-value-is-the-whole-value
date: 2026-10-09
---

# A raw value is the whole value

`IcalLine::raw_value_str` returned the first `,`-separated value of the first `;`-component: on an `RRULE` it read `FREQ=WEEKLY` and dropped `COUNT`, which is not what its name promises. Reading the callers decided it: the envelope values (`BEGIN`, `END`, `VERSION`) read the same either way, and the merge's component identity, reading `UID` and `RECURRENCE-ID`, was harmed by the split, a `UID` holding a `;` or `,` losing its tail.

So the name stays and the behaviour follows it. `raw_value` and `raw_value_str` return the whole raw value, still escaped, through a new `IcalValueNode::raw_bytes` that borrows a parsed value and joins a split one; the merge's private copy of the same read is folded into it. `raw_value` returns `Cow<[u8]>` rather than `&[u8]`, a breaking signature change, since a joined value cannot be borrowed. The `VERSION` read, where RFC 5545 3.7.4 allows `minver;maxver`, takes its first component explicitly, as it did implicitly before.

Capabilities moved: parsing.
