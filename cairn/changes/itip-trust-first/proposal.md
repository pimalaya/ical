---
cairn: change
id: itip-trust-first
status: draft
created: 2026-10-18
---

# iTIP messages: trust first, then supersession

## Why

The crate reads and writes `METHOD` and the scheduling properties but decides nothing about an iTIP message (RFC 5546) arriving by mail (iMIP, RFC 6047): whether to apply a `REQUEST`, a `CANCEL` or a `REPLY`, and whether it is newer than what the calendar holds. Every client applying invitations would write that judgement again, and the order of its two checks is where it goes wrong: a forged `REQUEST` with a high `SEQUENCE` passes a staleness check and replaces a real event.

pimdir's capabilities change left iMIP to "an implementation a sending source declares, deferred to the NLnet 2027 plan" (2026-10-03). This is the format half of it, I/O-free, so every client decides alike.

Borrowed from an external engine: reject on trust before looking at freshness, and order revisions by a key that names the instance.

## What

A pure decision over an iTIP calendar, the calendar the store holds for its `UID`, and the authenticated sender the caller supplies (this crate authenticates nothing: DKIM, DMARC or S/MIME is the mail layer's):

- **Trust first.** A `REQUEST`, `ADD` or `CANCEL` whose authenticated sender is not the `ORGANIZER`, and a `REPLY` whose sender is not the `ATTENDEE` it answers for, is refused before any revision is compared (the security considerations of RFC 5546 and RFC 6047). A refusal says which check failed.
- **Then supersession.** A revision is `(instance, SEQUENCE, DTSTAMP)`, the instance being the `UID` and the `RECURRENCE-ID` (none for the master): a message applies when its `SEQUENCE` is greater, or equal with a later `DTSTAMP`, than what is held for that instance (RFC 5546 §2.1.5).
- **A `REPLY` is ordered per attendee.** Its revision is kept per `(instance, attendee)`, so one attendee's stale reply never overwrites another's newer one, and an organizer's new `SEQUENCE` does not make every reply stale.
- The answer names what to do (apply, ignore as stale, refuse as untrusted) and, for an apply, the components it touches; producing the merged calendar stays the caller's, through the merge or a plain replacement.

## Open questions

- Whether "the sender is the organizer" compares `mailto:` addresses canonically (case, the `SENT-BY` parameter, RFC 5545 3.2.18) and how a delegate is admitted.
- `COUNTER`, `DECLINECOUNTER` and `REFRESH`: in the first version or later.
- Where the per-attendee revisions live between messages: in the held calendar's `ATTENDEE` parameters, or in state the caller keeps.
