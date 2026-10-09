---
cairn: change
id: rfc5545-contract-fixes
status: landed
created: 2026-10-06
---

# Four places where ical-rs is narrower or laxer than RFC 5545

## Why

calendula's field flags (calendula `2bc07e1`) build events through ical-rs and check them with its validator. Four gaps showed up; calendula works around each one today:

1. **The validator refuses what RFC 5545 allows.** `DTSTART` declares `allowed_values` of `DateTime` only (`src/prop/dtstart.rs`), likewise `DTEND`, `DUE` and `RECURRENCE-ID`, though each takes a `DATE` (3.8.2.2, 3.8.2.3, 3.8.2.4, 3.8.4.4). The same four, with `EXDATE` and `RDATE`, keep the default `COMMON_PARAMS` (VALUE, LANGUAGE, ALTREP), so their `TZID` is refused (3.8.2, 3.8.5). `ORGANIZER` refuses `CN`, `DIR` and `SENT-BY` (3.8.4.3) and the RFC 6638 and 7986 parameters `ATTENDEE` already accepts. Every whole-day, zoned or organized event fails the check. calendula filters these findings out (`is_spurious`, `src/shared/ical.rs`).
2. **The encoder never folds.** `IcalProp::encode` writes a line of any length, where RFC 5545 3.1 says lines SHOULD NOT exceed 75 octets; the fold helper is not public. calendula folds its own lines and reads them back through `push_raw`.
3. **Calendar addresses are text-escaped.** A `CAL-ADDRESS` is a URI (3.3.3), but it is encoded through `scalar_node` with text escaping, so a `,`, `;` or `\` in an address would gain a backslash. A `URI` value is written as is.
4. **`IcalDuration::seconds` is lenient.** It accepts forms outside the 3.3.6 grammar (`P1H`, the hour without `T`), so calendula validates `--duration` itself.

## What

1. Widen the contracts: `DATE` beside `DATE-TIME` on `DTSTART`, `DTEND`, `DUE`, `RECURRENCE-ID`; `TZID` on those four and on `EXDATE`, `RDATE`; `CN`, `DIR`, `SENT-BY`, `EMAIL` and the scheduling parameters on `ORGANIZER`. Check the other properties' contracts against RFC 5545 section 3.8 while there, and list any further gap.
2. Fold encoded lines at 75 octets, never inside a UTF-8 sequence (3.1), in `IcalProp::encode` and wherever a line is encoded; a decoded line keeps its recorded layout as today.
3. Encode a `CAL-ADDRESS` as a URI is, without text escaping.
4. Make `IcalDuration::seconds` (or a strict parse beside it) follow the 3.3.6 grammar exactly, so `P1H` reads as nothing; keep a lenient reading only where the decoder needs one for calendars in the wild, and say which.

Then calendula drops `is_spurious`, its own folding and its duration check.
