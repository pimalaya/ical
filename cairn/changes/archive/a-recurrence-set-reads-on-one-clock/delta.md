---
cairn: delta
change: a-recurrence-set-reads-on-one-clock
---

## ADDED Requirements

### Requirement: A set reads every time on its start's clock
Into recurrence. A set tells every time on its `DTSTART`'s clock (`IcalRecurSet::zone`), bringing a UTC `UNTIL`, an `RDATE` or `EXDATE` item and an override's `RECURRENCE-ID` and `DTSTART` written on another clock onto it through the zones that define both, read as written `DATE-TIME`s. `of_uid` finds the zones; `of_component_in`, `with_override_in` and `IcalRecurOverride::of_component` take them. Without them, times compare as written.

## MODIFIED Requirements

### Requirement: Civil expansion
In recurrence. The times a set compares are first told on one clock, a reading of the calendar rather than a change to how it expands.

## REMOVED Requirements
