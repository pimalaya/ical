---
cairn: log
change: one-matching-ladder-for-both-crates
landed: 2026-08-29
---

# One matching ladder, the same in both crates

vcard-rs adopted the property identity this crate landed earlier today, and the point of the exercise was that the two end up with one algorithm and two tables rather than two designs. Writing the shared ladder down showed two places where this crate did not say it.

The rungs were in the wrong order. Equality was consulted before identity, on the reasoning that an untouched property should pair with itself before anything renumbers it. That reasoning survives, but it is an argument for equality sitting above position, not above identity. In practice the two orders agree, because an identity is read off the value and equality implies equal values, so nothing here moves. What changes is that the code and the spec now read the same in both crates, and the rung above them both, an explicit synchronisation identity, is named as empty for iCalendar rather than absent without explanation. That is what vCard's `PID` occupies, and it is the one real difference between the two tables.

The comparison was on raw bytes, and that was a defect. RFC 3986 section 3.1 makes a URI scheme case-insensitive, and a mail host is case-insensitive too, so `MAILTO:Ada@Example.com` and `mailto:ada@example.com` are one person. Matching them by bytes missed the match: the base attendee had no counterpart, the side's line had no counterpart, and the merge reported a person leaving and a person arriving where an attendee had simply answered in a client that normalises its output. An identity is now lowercased for comparison.

The other half of that rule is what keeps it honest. Only the comparison normalises. The line goes back on the wire with the bytes the side that wrote it wrote, so a side that did rewrite the case of a scheme has changed the value and that change lands like any other, and a line nobody touched still comes out byte for byte. Compare on raw bytes and a match is missed; write back the normalised form and the byte fidelity the crate exists for is gone.

One test states it, and it fails without the fix: a base attendee written `MAILTO:Ada@Example.com`, one side adding a `CN` and the other lowercasing the whole address and accepting. One attendee, carrying the answer.

Capabilities moved: merge ("Property identity" MODIFIED, now stating the ladder and its empty first rung; "Matching normalises, writing is exact" ADDED).
