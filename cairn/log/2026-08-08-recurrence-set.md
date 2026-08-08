---
cairn: log
change: recurrence-set
landed: 2026-08-08
---

# The recurrence set, not one rule

`recur::set::IcalRecurSet` answers the question a client actually asks: when does this event happen? `DTSTART` plus every `RRULE` and `RDATE`, minus every `EXDATE` and `EXRULE`, with the `RECURRENCE-ID` overrides applied. `of_component` reads it off a decoded component, `of_uid` collects a series and its overrides across a whole calendar, and `expand` walks it.

The walk is a k-way merge with no buffer: one lazy stream per rule, one sorted list for the literal dates, the exclusions applied as candidates come past (an `EXDATE` is a binary search, an `EXRULE` is another lazy stream advanced in step). Taking a thousand occurrences from an unbounded daily rule does a thousand occurrences of work.

Two times per occurrence, which is the part worth stating plainly. The **identity** is where the rules put it, and it is what a `RECURRENCE-ID` names and an `EXDATE` removes. The **start** is when it happens, which is the identity unless an override moved it. Occurrences come out in identity order, so an override that moves an instance is emitted in the place of the instance it replaces and its start can fall out of order. That is the price of not buffering, it is documented rather than hidden, and a caller who needs starts in order sorts a window, which is a decision about a window rather than about the walk. `RANGE=THISANDFUTURE` shifts the named instance and every later one by the same offset; the latest such override in force wins rather than compounding. An override naming an instance no rule generates is still an instance, since its identity joins the merge as a source of its own.

Getting there needed a fix in the decoded model. `RDATE` and `EXDATE` are *lists* on the wire (RFC 5545 3.8.5.1 and 3.8.5.2), and both were decoding as a single `DATE-TIME`, so `RDATE:20260103T120000,20260104T120000` silently lost its second date. They now decode as `IcalValueKind::DateTimeList` into `IcalDateTimeList`, mirroring the `TextList` that `CATEGORIES` and `RESOURCES` already use, with each item kept as raw text so a period item keeps its `start/end` form. Their spec pins the list kind whatever `VALUE` declares, since a declared `DATE`, `DATE-TIME` or `PERIOD` describes each *item*, not the value as a whole.

`IcalRecurDateTime` gained `seconds` and `from_seconds`, the civil arithmetic the override shifts need, and the timezone work next in the backlog will want the same pair.

Eleven tests cover it: the RFC combination of rule, extra date and exception, several rules merging, a multi-valued `RDATE`, a period `RDATE`, an `EXRULE`, laziness over a thousand occurrences, a single override, a moved tail, an override of an instance no rule generates, a component with no recurrence at all, and a `VTODO`, which expands the same way a `VEVENT` does.

Capabilities moved: `recurrence` (ADDED: the recurrence set of a component, set expansion stays lazy, overrides replace instances); `decoded-model` gains the list value kind through `parsing`'s vocabulary rather than a requirement of its own.
