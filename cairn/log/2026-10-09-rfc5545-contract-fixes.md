---
cairn: log
change: rfc5545-contract-fixes
date: 2026-10-09
---

# Four places where ical-rs was narrower or laxer than RFC 5545

calendula's field flags build events through ical-rs and check them with its validator, and four gaps showed up, each worked around on calendula's side.

**The contracts refused what the RFCs allow.** `DTSTART`, `DTEND`, `DUE` and `RECURRENCE-ID` declared `DATE-TIME` alone, so every whole-day event failed the check, and they, `EXDATE` and `RDATE` kept the default `VALUE`, `LANGUAGE`, `ALTREP` set, so every zoned one did too. They now take `DATE` beside `DATE-TIME` and `VALUE` plus `TZID` (and `RANGE` on `RECURRENCE-ID`), written out exactly as RFC 5545 states them. Written out exactly, they no longer take the `LANGUAGE` and `ALTREP` the default set gave them, which the RFC gives none of them. `ORGANIZER` takes `CN`, `DIR`, `SENT-BY`, `LANGUAGE`, the RFC 7986 `EMAIL` and the three RFC 6638 scheduling parameters, as `ATTENDEE` already did.

Auditing the rest of section 3.8, and the extensions the crate covers, found more of the same kind, all fixed:

- `ATTACH` refused its inline form: `BINARY` value, `FMTTYPE` and `ENCODING` (3.8.1.1).
- `FREEBUSY` refused `FBTYPE` (3.8.2.6).
- `TRIGGER` refused its absolute form, `VALUE=DATE-TIME`, and `RELATED` (3.8.6.3).
- `DESCRIPTION` was single-valued, which refused a `VJOURNAL` carrying several (3.6.3) and a `VCALENDAR` carrying one per language (RFC 7986 5.2). It is repeatable now; the cardinality is one number for every component, so a second description in a `VEVENT` passes the check.
- `IMAGE` refused `BINARY`, `FMTTYPE`, `ENCODING` and `DISPLAY` (RFC 7986 5.10).
- `CONFERENCE` refused `FEATURE` and `LABEL` (RFC 7986 5.11).
- `LINK` refused `LANGUAGE`, `RELATED-TO` a `URI` value (RFC 9253).
- `STRUCTURED-DATA` refused `BINARY` and `URI` values and `FMTTYPE`, `SCHEMA`, `ENCODING`; `STYLED-DESCRIPTION` refused a `URI` value and `FMTTYPE`, `DERIVED` (RFC 9073 6.5, 6.6).

Left as found, and named in the spec: the default parameter set lets `LANGUAGE` and `ALTREP` onto properties the RFC gives neither (`PRIORITY`, `UID`, the date-times not listed above), and `VALUE` onto every property. Narrowing it would refuse calendars the check passes today, which wants a change of its own.

**The encoder never folded.** A line built from the model, or one an edit had lengthened out of its recorded shape, went out on one physical line of any length, where RFC 5545 3.1 says 75 octets. The wire shape now tells a built line (never sealed) from a parsed one, and a line with no shape that still fits it is folded at 75 octets, the continuation space counted, never inside a UTF-8 sequence, with the line's own break. A parsed line keeps its layout byte for byte, a long unfolded one included. A `QUOTED-PRINTABLE` line is never folded, since a fold after one of its `=` reads back as a soft break.

**Calendar addresses were text-escaped.** A `CAL-ADDRESS` is a URI (3.3.3), and it went through the text escape, so a `,`, `;` or `\` gained a backslash. It is written as a URI is now, on both write paths: the model's encode, and an in-place edit through the value cursor, whose `set_text` and `set_bytes` now ask the line's spec for its value type (the Android phone mirror patches `ORGANIZER`, `ATTENDEE` and `URL` that way). Writing it raw would have let a line break end its line, so a URI and an address both percent-encode one now (RFC 3986 2.1), which also closed that hole for `URI` values. Reading is unchanged and lenient: a backslash in a URI still reads as the text escape it most likely is.

**The duration grammar had no checker.** A first pass made `IcalDuration::seconds` strict; review turned that back, since it breaks liberal-in: every reader computing an end from a decoded calendar (the JSCalendar import here, the Android agenda and phone mirror downstream) would lose one on a wild `P1H` or `p1d`. `seconds` stays a lenient reader of what calendars in the wild write, now in either case, with a fraction of second dropped and its arithmetic overflow-checked, which also makes the RFC 8984 reading the JSCalendar import needs (`P1W2D`) its own. The strict 3.3.6 grammar became what the validator checks: a new `IcalValidateError::Duration` reports a `DURATION`, a relative `TRIGGER`, a `REFRESH-INTERVAL` or a `GAP` parameter outside it (a week stands alone, a `T` names a unit, units in order, a minute between an hour and a second), so calendula's `--duration` check is the validator's. A period's duration is not checked yet. `IcalDuration::from_seconds` wrote one of the refused forms for 3601 seconds (`PT1H1S`) and now spells the minute.

In-crate duration readers after the change: the JSCalendar import's end computation reads leniently (`seconds`), the validator reads strictly (the private grammar check), and the JSCalendar export writes strictly (`from_seconds`). Recurrence expansion, the time zones and the merge read no duration.

Capabilities moved: conformance, parsing, decoded-model. The calendula side (dropping `is_spurious`, its folding and its duration check) and the release are left to follow.
