---
cairn: change
id: ours-wins
status: landed
created: 2026-08-30
---

# Hard-code ours over theirs

## Why

`prefer` let a caller say which side wins a collision, apart from which side supplies the baseline bytes. The spec justified that split in as many words: authority sat on the replayed side, so a caller that needed its own edit judged had to put that edit on the right, and without a separate preference it would have paid for being judged by losing every collision.

Authority is gone. The justification went with it, and what is left is a field every caller in the ecosystem sets to the same value. tcard, tcal and neverest all pass `Left`; nothing passes `Right`.

Git's vocabulary already names this and everybody knows it: the side being merged into is `ours` and it wins, the side being merged in is `theirs`. A knob with one setting is worse than the convention it obscures.

## What

Remove `prefer` and `IcalMergeSide`. The left side is `ours`: the merged calendar is built from its bytes and keeps its value where both sides wrote one. The right side is `theirs`. Every collision is still reported, so a caller wanting the other value puts it to somebody rather than asking the merge to guess.

Done when the field and the enum are gone, the spec says the rule rather than the choice, and every caller has stopped stating it.

## Consequence

One mechanism dies with it. An addition could only displace the other side's while the right side was able to win, so `displaces`, `both_added` and the replace-where-it-stood paths were unreachable the moment the preference went. They are removed rather than left to look live.
