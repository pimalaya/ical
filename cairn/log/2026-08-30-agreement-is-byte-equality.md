---
cairn: log
change: agreement-is-byte-equality
landed: 2026-08-30
---

# Agreement is byte equality

One defect shape has now been fixed at four sites in three days: comparing the decoded form of something whose decode is not injective. Values (`value_eq`), URIs and parameters (`param_eq`) were the first three. Action-level agreement was the fourth and the last.

`agrees` required raw byte equality for a property added and for a component added, and its `_ => true` arm required nothing at all for everything else. `\N` and `\n` both unescape to a line break (RFC 5545 section 3.3.11), so a left side writing `SUMMARY:b\nc` and a right side writing `SUMMARY:b\Nc` produced equal `ValueChanged` actions, the right side's act was skipped as already made, and the two sides were told they had agreed on bytes they had not agreed on.

`agrees` is now `same_change` over the decoded action followed by `wrote_alike` over the bytes the act itself put on the wire: the component or the line an addition put there, the value a change wrote, the item a list gained, the parameter a side wrote. An act that only takes something away wrote no bytes, and what it names lives in the base both sides share, so the act itself settles it. `IcalValueNode::raw_list` is the raw twin of `decode_list`, which is what lets an item be weighed as its own leaf rather than as the whole value it sits in.

The merged bytes did not move. A refused agreement is judged like any other act, meets the left side's on the same field, and is reported as `Divergent` with `applies` false, so the left side keeps its value. Only the report gains an entry, which is the whole point: the difference is said out loud instead of vanishing.

The crate also had no order-insensitivity anywhere, while vcard-rs has compared the items of `TYPE` and `PID` as sets from the start. iCalendar has unordered list parameters too, and nothing said so: `DELEGATED-FROM` and `DELEGATED-TO` (sections 3.2.4 and 3.2.5), `MEMBER` (section 3.2.11) and `FEATURE` (RFC 7986 section 6.3). `same_param` and `sorted` are the twins of vcard-rs's, `unordered` names the four kinds once, and `param_alike` compares an unordered parameter's raw values as a set too, so the exception holds on the wire as well as in the model. Without that second half the byte rule would have undone the first: `DELEGATED-TO="a","b"` and `DELEGATED-TO="b","a"` are different bytes.

One consequence had to be fixed alongside. Replaying a list item wrote the whole list back whether or not anything changed, and writing a list back escapes every item afresh, so a replay that changed nothing would have spelled the baseline side's own items the canonical way. It now writes back only where the item really joins or leaves, which is what vcard-rs's targeted `push_value` and `remove_value_at` already did.

Spec updated: `merge` (MODIFIED: agreement is not a collision; ADDED: a list value is written back only when it changes).
