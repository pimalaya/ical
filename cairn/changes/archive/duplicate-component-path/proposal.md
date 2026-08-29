---
cairn: change
id: duplicate-component-path
status: landed
created: 2026-08-29
---

# Two components at one path are two components

## Why

A calendar may hold one `UID` twice with no `RECURRENCE-ID` telling the two apart. RFC 5545 does not allow it, but real files and truncated downloads carry it, and the merge has to answer for what it is given rather than refuse it.

The diff matched every base component against the first side component sharing its path, so both duplicates were compared with the same one and the difference between the duplicates was reported as a change each side made. Merging a calendar with itself then reported a collision per duplicate, each naming a value nobody wrote, which the merge fuzz target found on a holidays fixture.

## What

Each side component is matched once. A base component takes the first side component at its path that no earlier base component has taken, so two duplicates pair with two duplicates, in the order they are written, which is the only ordering available where nothing tells them apart.

## What this does not fix

The replay still addresses such a component by its path alone, so an action about the second of two may land on the first. What no addressing can tell apart the merge does not claim to: it pairs them in the order they are written and reports what it cannot settle. Giving a component an occurrence beside its `UID` would fix it and would change the public path shape, which is not worth it for input RFC 5545 3.8.4.7 does not allow.
