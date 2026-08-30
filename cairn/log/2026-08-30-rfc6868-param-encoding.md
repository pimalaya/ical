---
cairn: log
change: rfc6868-param-encoding
date: 2026-08-30
---

# A parameter is not a text value

The parameter codec had been pointed at the text unescaper, and its own doc comment said so: the RFC 5545 3.3.11 escapes were "the default used wherever the escaping mode is not version-specific (parameters, the version-blind lens path)". Section 3.2 gives a parameter value no escapes at all, which is the whole reason RFC 6868 exists, so the crate was wrong twice at once. A backslash a parameter legitimately carried was eaten on the way in and could not be written back, and a real `^n` or `^'` from a conforming producer reached the caller with its encoding showing.

Both halves are now RFC 6868. `unescape_param` resolves `^n`, `^^` and `^'` and leaves every other caret sequence exactly as written, which section 3.1 requires rather than merely permits, and `escape_param` writes them back. Neither touches a backslash.

Two decisions shaped the rest.

The rules are keyed on the version, not applied everywhere. RFC 6868 updates RFC 5545 and nothing earlier, so a vCalendar 1.0 caret is a literal caret and resolving it would corrupt the value. `Escaper::has_param_encoding` is the switch. That forced the seam the parameter side had never had: `IcalParamNode` now carries an `escaper`, stamped by `stamp_escaper` alongside the value nodes once `VERSION` is known, and `IcalParam::encode` and `IcalParamLens::encode` take the target mode the way `IcalProp::encode` and the `Codec` trait already did. Thirty-three lens modules moved with it.

The encoder leaves a quoted value's own delimiters alone. The decoded model holds a parameter exactly as the wire spelled it, the surrounding double quotes included, so encoding the pair would rewrite every quoted URI as `^'...^'` and no `ALTREP` would survive its own round trip. A quoted value is encoded inside its quotes instead, which keeps the RFC's motivating case working: a `CN` carrying a quote mid-value still writes `^'`.

The transition is invisible, as Postel asks. A parameter with neither a caret nor a backslash means the same under both readings, and that is every parameter of every fixture: the rfc, vcalendar, libical, ical4j and icaljs corpora all classify exactly as they did, and the calcard cross-check is unchanged. Nothing moved.

A second defect came out of the same reading. The merge compared parameters in their decoded form, and a single-valued parameter decodes its first value alone, so `CN=Ada,Lovelace` against `CN=Ada,Byron` compared equal: no action was reported, and the right side's edit was dropped without a word. Parameters now go through `param_eq` on their raw nodes, value by value, exactly as `value_eq` already did for values, falling back to raw bytes where two calendars of different versions share no decoding. `nth_param` returns a position now, so the decoded list and its raw nodes are addressed by the same index.

Capabilities moved: decoded-model, merge.
