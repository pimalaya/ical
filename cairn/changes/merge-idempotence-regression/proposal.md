---
cairn: change
id: merge-idempotence-regression
status: active
created: 2026-10-18
---

# The merge idempotence crash input as a regression test

## Why

The Android audit's S3 crash input ical-merge-idempotence.bin (225 bytes, the merge fuzz target's corpus seed 52b059d4, written 2026-08-29) broke the idempotence oracle: merging two identical sides against a base reported a collision. hostile-input-bounds (2026-10-11, caa1422) fixed the two agreement bugs behind it, and a probe of the input now reports no conflict. tests/merge.rs gained three cases with that fix, none of them this input's `X-TIME;VALUE=TIME` beside `X-TIMEZ;VALUE=TIME` with a `Z`, a floating and a UTC time side by side.

The fuzz corpus is not committed, and the audit folder holding the input is about to be deleted, so the bytes are kept here.

## The input

Three calendars separated by a NUL byte, as fuzz/fuzz_targets/merge.rs splits them: base, left, right. Every line ends on a bare LF and each part ends on an empty line. The authoritative bytes, as hex:

```
55453d54494d453a3233303030350a582d54494d455a3b56414c557e4d453d543249453a33303030305a0a454e443a5643414c454e4441520a0a00424547494e3a5643414c454e4441520a582d54494d453b56414c55453d54494d453a3233303030300a582d54494d455a3b56414c55453d54494d453a3233303030305a0a454e443a5643414c454e4441520a0a00424547494e3a5643414c454e4441520a582d54494d453b56414c55453d544d4d453a7633303030300a582d54494d455a3b56414c55453d54494d453a3233303030305a0a454e443a5643414c454441520a0a
```

The same, readable. Base (no `BEGIN`, a broken first line):

```
UE=TIME:230005
X-TIMEZ;VALU~ME=T2IE:30000Z
END:VCALENDAR

```

Left:

```
BEGIN:VCALENDAR
X-TIME;VALUE=TIME:230000
X-TIMEZ;VALUE=TIME:230000Z
END:VCALENDAR

```

Right (`VALUE=TMME`, and `END:VCALEDAR` misspelt):

```
BEGIN:VCALENDAR
X-TIME;VALUE=TMME:v30000
X-TIMEZ;VALUE=TIME:230000Z
END:VCALEDAR

```

## What

One case in tests/merge.rs, the bytes inlined: `IcalMerge { base, left, right: left }` yields the left calendar byte for byte and no conflict. The same case also runs the target's other oracles on all three parts: the merged calendar reparses to its own bytes, a calendar merged with itself is unchanged and quiet, and every conflict of `base, left, right` names an action the right side took.
