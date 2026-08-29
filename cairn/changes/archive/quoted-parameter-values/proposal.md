---
cairn: change
id: quoted-parameter-values
status: landed
created: 2026-08-29
---

# A quoted parameter value may carry a colon and a semicolon

## Why

RFC 5545 section 3.2 gives `param-value = paramtext / quoted-string`, and a `quoted-string` holds any `QSAFE-CHAR`, which is every character but a control and a double quote, so `:` and `;` are both legal inside a quoted parameter value. Two places in the line splitter ignore that: the value separator is found with the first `:` anywhere in the line, and the head is split on every `;`.

The RFC's own section 3.2.1 example is the casualty:

    DESCRIPTION;ALTREP="cid:part1.0001@example.org":Meeting notes

parses to a parameter `ALTREP` holding `"cid` and a value reading `part1.0001@example.org":Meeting notes`. `ATTENDEE;DIR="ldap://host:389/cn=x"` breaks the same way, and the corpus already carries such lines: `ALTREP="CID:<...>"` in the ical4j calendars, `DELEGATED-TO="mailto:..."` in the icaljs ones. Round-tripping hides all of it, since every piece is written back verbatim.

What is wrong is the parsed structure, so the three-way merge, which diffs that structure, reads a parameter edit as an edit of the value it was folded into and reports a collision where there is none.

## What

Both scans become quote aware: the value separator is the first `:` outside a double-quoted run, and the head splits on the first `;` outside one. The quoted-printable head probe uses the same separator, so a quoted colon no longer moves it either.

The parameter value splitter already skips commas inside quotes, so it needs nothing.

## Judgement call, for review

**An unbalanced quote falls back to the quote-blind scan.** Naive quote tracking would let one stray `"` swallow the rest of a junk line and turn a parseable line into `MissingPropertyColon`. When no colon sits outside quotes, the parser takes the first colon anywhere instead, which keeps the liberal parse the crate promises. This mirrors the twin crate vcard-rs, which took the same decision today.
