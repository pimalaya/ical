---
cairn: tasks
change: jcal
---

- [x] Add the jcal feature with serde_json as its only dependency
- [x] Encode a component as the two-array jCal form, properties then sub-components
- [x] Encode each property as name, params, type, value, with the RFC 7265 value spellings
- [x] Decode jCal back to the model, keeping unknown names, params and types
- [x] Round-trip the RFC 7265 examples both ways as tests
