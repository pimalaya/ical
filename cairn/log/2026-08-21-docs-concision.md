---
cairn: log
change: docs-concision
landed: 2026-08-21
---

# Trim the documentation and fix what went stale

A sweep over every documentation surface (the lib.rs header, the module headers, the item docs, and the markdown files) for concision and for statements the code had outgrown. No behaviour, no public API and no capability moved: this is prose only.

**Stale statements.** The value cursor still carried vcard-rs's header word for word: it claimed to serve "every property lens but `N`", named `ADR`, `GENDER`, `ORG` and `CLIENTPIDMAP` as the structured kinds, and linked an `IcalNCursor` at a module that does not exist. This crate has one cursor type and two structured values, `GEO` and `REQUEST-STATUS`. Fourteen more item docs called a calendar a "card", inherited the same way. The strict layer said it checked calendars against RFC 6350, which is vCard; it is RFC 5545. Validation gated recurrence-rule checking on a `recur` feature that no longer exists (the check is unconditional), and CONTRIBUTING listed that feature in the build matrix while omitting jcal and jscalendar. jcal.rs called JSCalendar "next in the backlog" two releases after it landed, and the RECUR value said typed decoding was "deferred to a future addition" when `IcalRecurRule::parse` does it. The lib.rs header claimed the only dependencies were the content-decoding crates, forgetting memchr and serde_json, and never mentioned the merge. SECURITY.md still supported 0.0.x. fuzz/README described fuzzing the vCard parser over a corpus of cards.

**Concision.** The lib.rs header keeps its sections but states each in fewer words, and its feature list now names the crate each feature pulls and what each implies. The module headers that had grown past three paragraphs (cst, merge, jscalendar, recur, recur/set, timezone, wire, node, line, codec, ical, component, valid) are back to a summary line plus one or two paragraphs. The eleven decoded-value modules each closed on the same sentence about being pure data whose wire name lives on `IcalProp::name`; that is stated once in the parent value.rs now.

**Consistency.** Fifteen lens headers named their property or parameter by its Rust marker (`LAST_MODIFIED`, `DELEGATED_FROM`) rather than its wire spelling, against every other lens in the crate; they now read `LAST-MODIFIED` and `DELEGATED-FROM`. Six value-codec titles started lowercase. Sixteen doc sentences read "a `Ical...`" rather than "an".
