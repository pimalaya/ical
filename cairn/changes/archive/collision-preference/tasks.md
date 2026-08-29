---
cairn: tasks
change: collision-preference
---

- [x] An `IcalMergeSide` enum naming the two sides, defaulting to the left
- [x] A collision preference field on `IcalMerge`, carrying it
- [x] `judge` decides a both-wrote-a-value collision by the preference
- [x] The removal-against-update rule stays fixed, whatever the preference
- [x] The report names both actions whichever side won
- [x] Document on `IcalMerge` that the baseline side and the winning side are now separate questions
- [x] Test: with the preference on the right, a same-property collision carries the right value
- [x] Test: with the preference on the left, the same merge is unchanged from today
- [x] Test: an update still beats a removal under both preferences
- [x] Test: a property only one side touched is unaffected by the preference
- [x] Test: an untouched folded line still comes out byte for byte under both preferences
- [x] Test: a right-side action refused for want of authority stays refused under both preferences
