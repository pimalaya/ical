---
cairn: change
id: jscalendar
status: landed
created: 2026-08-08
---

# JSCalendar, RFC 8984

## Why

The analogue of vcard-rs's `jscontact`, and what a modern client actually speaks. JSCalendar is the data model JMAP Calendars carries, so a crate that stops at iCalendar cannot serve one.

## What

A bidirectional conversion built on jCal, in the same relationship the JSContact conversion has to jCard: what the mapping cannot express is kept in an escape hatch rather than dropped. This is larger than jCal because JSCalendar is a genuine re-modelling, not a re-encoding: recurrence rules, participants, alerts and localisations all change shape.

Done when the RFC 8984 examples convert both ways and the conversion is lossless for everything the RFC maps.
