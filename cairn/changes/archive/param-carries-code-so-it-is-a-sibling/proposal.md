---
cairn: change
id: param-carries-code-so-it-is-a-sibling
status: landed
created: 2026-08-29
---

# The param module carries code, so it is a sibling file

## Why

The syntax-side `param` module holds the `COMMON_PARAMS` const the property spec reads, next to its per-parameter submodules. The Pimalaya layout rule (naming-002) says a pure aggregator, holding only module declarations and re-exports, lives in foo/mod.rs, and a module with code of its own lives in a sibling foo.rs beside its foo/ folder.

This one has code, so it is the second shape and is written as the first. vcard-rs, whose `param` holds the same const, is already a sibling file, and the two crates are read against each other.

## What

Move src/tree/param/mod.rs to src/tree/param.rs. Nothing else moves: the module path, the const and every submodule are unchanged.
