---
cairn: tasks
change: rfc5545-contract-fixes
---

# Tasks

- [ ] Contracts: `DATE` and `TZID` on the time properties, `ORGANIZER` parameters; audit section 3.8 for other gaps.
- [ ] Folding at 75 octets on encode, UTF-8 safe; tests on a long text line and a multibyte boundary.
- [ ] `CAL-ADDRESS` encoded as a URI; test with `,` and `;` in an address.
- [ ] Strict duration grammar; tests on `P1H`, `PT1H`, `P1W`, `-PT15M`, `P1DT2H`.
- [ ] Fold into the spec (conformance, decoded-model or parsing as fits), log, changelog, release.
- [ ] Then in calendula: remove `is_spurious`, the local folding and the duration check; bump ical-rs.
