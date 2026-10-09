---
cairn: tasks
change: rfc5545-contract-fixes
---

# Tasks

- [x] Contracts: `DATE` and `TZID` on the time properties, `ORGANIZER` parameters; audit section 3.8 for other gaps.
- [x] Folding at 75 octets on encode, UTF-8 safe; tests on a long text line and a multibyte boundary.
- [x] `CAL-ADDRESS` encoded as a URI; test with `,` and `;` in an address.
- [x] Strict duration grammar, checked by the validator while `IcalDuration::seconds` stays a lenient reader; tests on `P1H`, `PT1H`, `P1W`, `-PT15M`, `P1DT2H`.
- [x] The value cursor encodes a URI and a calendar user address as they are on an in-place edit too.
- [x] Fold into the spec (conformance, decoded-model and parsing), log, changelog.
- [ ] Release.
- [ ] Then in calendula: remove `is_spurious`, the local folding and the duration check; bump ical-rs.
