---
cairn: tasks
change: a-folded-line-is-stripped-once-assembled
---

- [ ] A regression test that fails on the current code: the calendar of the proposal decoded, written and reparsed keeps `X-A`, and `IcalLine::take` on `"   \r\n  A:b\r\n"` names the line `A` and serializes back to its input
- [ ] src/tree/line.rs: strip the whitespace a continuation brings to an empty logical line, recorded as skipped bytes, in the folded and the `QUOTED-PRINTABLE` paths (vcard-rs's `push_content`); keep the first speed pass's one-copy join for every other line
- [ ] The corpus tests unchanged; the parse and merge fuzz targets for ten minutes each
- [ ] Fold the delta into spec/parsing.md, log, CHANGELOG
