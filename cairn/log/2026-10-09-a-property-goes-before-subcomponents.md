---
cairn: log
change: a-property-goes-before-subcomponents
date: 2026-10-09
---

# A property goes before the subcomponents

A property pushed onto a component went after its last item, so on a `VEVENT` holding a `VALARM` it followed `END:VALARM`, which RFC 5545 3.6 does not allow: the component's properties come first. `IcalCst::push`, `IcalCst::push_raw` and the merge's added properties all did it, and the Android app re-inserted every pushed item by hand.

`IcalCst::push_line` (tree/cst.rs) is the primitive: it inserts a property line before the first subcomponent, or at the end when there is none, counting a stashed subcomponent kept as raw lines from its `BEGIN`. `push` encodes through it; `push_raw` places its property lines there in order and appends the lines from a `BEGIN` to its `END` after everything else; the merge's replay adds through it. Inserting moves nothing already there, so every parsed line, blank lines recorded before a `BEGIN` included, keeps its bytes.

Capabilities moved: parsing.
