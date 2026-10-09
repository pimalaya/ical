---
cairn: change
id: a-raw-value-is-the-whole-value
status: landed
created: 2026-10-09
---

# A raw value is the whole value

## Why

`IcalLine::raw_value_str` returned the first `,`-separated value of the first `;`-component, so on an `RRULE` it read `FREQ=WEEKLY` and dropped `COUNT`, though its name promises the raw value. Its callers in the crate are the `BEGIN`, `END` and `VERSION` envelope values, where first and whole agree, and the merge's component identity, which reads the `UID` and `RECURRENCE-ID` and was the one place the split could do harm, cutting a `UID` at a `;` or `,`.

## What

`raw_value` and `raw_value_str` return the whole raw value, still escaped, borrowed when the line is as parsed. `raw_value` returns a `Cow<[u8]>` instead of a `&[u8]`, since a value split by an edit has to be joined. The one caller that wanted only a part, the `VERSION` read (RFC 5545 3.7.4 allows `minver;maxver`), takes its first component explicitly.
