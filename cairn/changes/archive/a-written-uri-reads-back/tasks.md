---
cairn: tasks
change: a-written-uri-reads-back
---

# Tasks

- [x] `verbatim_node` writes `\` as `%5C`, which the encode and the cursor both go through.
- [x] Tests: an address encoded with `\`, and an in-place edit of `ORGANIZER` and `URL` carrying `,` `;` `\` whose value reads back and writes back the same bytes.
- [x] Fold into decoded-model and parsing, log, changelog.
