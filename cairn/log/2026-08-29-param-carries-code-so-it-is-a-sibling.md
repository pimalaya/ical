---
cairn: log
change: param-carries-code-so-it-is-a-sibling
landed: 2026-08-29
---

# The param module carries code, so it is a sibling file

src/tree/param/mod.rs is now src/tree/param.rs. No code moved with it: the module path, the `COMMON_PARAMS` const the property spec reads, and every per-parameter submodule are what they were.

The rule is naming-002, and it is content-based rather than a matter of taste. A module holding only declarations and re-exports is an aggregator and lives in foo/mod.rs; a module holding code of its own is a sibling foo.rs beside its foo/ folder, so a reader looking for the code finds a file rather than a directory to open. `param` holds a const, so it is the second kind.

Every other module pair in the crate was checked against the same test rather than only the one the twin flagged. src/tree/codec.rs is already a sibling and already carries the `Codec` trait. src/tree/prop/mod.rs and src/tree/component/mod.rs re-export their spec function and declare their submodules, which is an aggregator by the rule's own words, so both stay. src/tree/value/mod.rs, src/tree/ical/mod.rs and src/tree/mod.rs declare and nothing more.

vcard-rs, whose `param` holds the same const in a sibling file, needed no move. That is what made this visible: the two crates are read against each other, and one crate's `param/` against the other's `param.rs` costs a reader a question with no answer behind it.

No capability moved: this is layout, and the spec describes behaviour.
