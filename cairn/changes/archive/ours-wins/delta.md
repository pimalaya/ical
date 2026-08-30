---
cairn: delta
change: ours-wins
---

## MODIFIED Requirements

### Requirement: Ours wins, and the collision is still reported

The left side SHALL be `ours` and the right side `theirs`, in git's sense. The merged calendar SHALL be built from the left side's bytes, and where both sides changed one property to different things it SHALL carry the left side's value. Neither is a caller's to choose.

### Requirement: An addition that loses does not join the one that beat it

Where both sides added a property or a component the base lacked, the merged calendar SHALL hold the left side's alone and report the collision. The right side's addition SHALL NOT be written beside it.

## REMOVED Requirements

### Requirement: The winning side is chosen, not implied

Removed. A caller can no longer say which side wins a collision. The rule is the convention git already names, and the split it replaced was justified by an authority model the crate no longer has.
