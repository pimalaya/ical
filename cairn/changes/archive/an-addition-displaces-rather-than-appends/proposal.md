---
cairn: change
id: an-addition-displaces-rather-than-appends
status: landed
created: 2026-08-29
---

# The winner of a both-sides-added collision replaces the loser

## Why

Where both sides add a property or a component the base lacked, the collision is reported and the preferred side is meant to win. Under the right preference it does not win, it joins: the replay pushes onto the component's items and there is nothing there for it to replace, so both survive.

No value is lost, but two costs are real. The merged calendar can carry two `LOCATION` lines in one `VEVENT`, which RFC 5545 3.6.1 forbids and this crate's own `validate` refuses, so two of its entry points contradict each other. And `merge(base, x, x)` under the right preference returns `x` with every addition present twice, so merging a calendar with itself is a mutation and a synchronisation engine that re-merges after a failed write doubles the additions each time.

## What

An addition that wins a collision displaces the addition it beat. The replay is told which left-side action the right-side one is beating, and takes that line or that component out before putting its own in. An uncontested addition still appends.

Idempotence follows: two byte-identical sides produce two equal actions, which no longer collide, and the winner replaces a line identical to itself.
