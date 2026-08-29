---
cairn: log
change: quoted-parameter-values
landed: 2026-08-29
---

# A quoted parameter value may carry a colon and a semicolon

The line splitter cut the head at the first `:` anywhere and split parameters on every `;`, ignoring RFC 5545 section 3.2, which lets a double-quoted parameter value hold both. The RFC's own `DESCRIPTION;ALTREP="cid:part1.0001@example.org":Meeting notes` parsed to a parameter holding `"cid` and a value reading `part1.0001@example.org":Meeting notes`; `ATTENDEE;DIR="ldap://host:389/cn=x"` broke the same way, and the corpus carries such lines already, in the ical4j and icaljs calendars. Round-tripping hid it, since every piece is written back verbatim.

What it cost was the parsed structure: the three-way merge diffs that structure, so an edit to a quoted parameter read as an edit of the value it had been folded into, and two sides editing the parameter and the value of one property collided instead of merging.

Both scans are now quote aware, with a fallback to the quote-blind scan when an unbalanced quote leaves no colon outside quotes, so a junk line still parses rather than failing. The parameter value splitter already skipped commas inside quotes and is unchanged.

Spec updated: `parsing` (ADDED: a quoted parameter value is opaque).
