---
cairn: log
change: building-a-calendar-from-another-representation
landed: 2026-10-01
---

# Building a calendar from another representation

The iCalendar side of vcard-rs 0.5.1's builders. The Google event projection, which moved from Calendula into io-gcal, carried private helpers to start an empty component and mint text properties; the Microsoft Graph event projection to come needs the same, so they moved here rather than be copied.

`IcalCst::empty` (tree/cst.rs) starts a component with only its BEGIN/END envelope, beside `IcalCst::v2`, which starts a whole calendar; `component` was taken by the getter. `IcalProp::text` (prop.rs) builds a text property for a known kind or an `X-` name, with the same signature as `VcardProp::text`. A third helper, which only filled `IcalProp`'s public fields, became struct literals in io-gcal instead of API.

Capability moved: **decoded-model** (building a calendar from another representation).
