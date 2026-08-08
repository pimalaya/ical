---
cairn: change
id: jcal
status: landed
created: 2026-08-08
---

# jCal, RFC 7265

## Why

The direct analogue of vcard-rs's `jcard` feature. A JSON spelling of a calendar is what a web client or a JMAP-adjacent store wants, and RFC 8984 (JSCalendar) needs it as its escape hatch, exactly as RFC 9555 needs jCard.

## What

A codec on the decoded model behind an opt-in feature, with `serde_json` as the only dependency and a raw `Value` at the boundary rather than serde impls on any calendar type: one type can have two JSON spellings, so serde, which keys one representation per type, is the wrong tool. The component tree makes this slightly more work than jCard, since jCal nests properties and sub-components as two arrays per component.

Done when a calendar round-trips model to jCal to model, with the RFC 7265 examples as tests.
