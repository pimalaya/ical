---
cairn: change
id: tzdb-pinned-and-versioned
status: landed
created: 2026-10-18
---

# The bundled time zone database names its version and resolves what a calendar omits

## Why

An occurrence of a zoned series is an instant only through a time zone's rules, and those rules change: a country moving its daylight-saving dates moves every future occurrence written under its `TZID`. A consumer that materialises occurrences (a search index, an agenda cache, a free-busy view) has no way to know its rows were computed under rules since replaced, so a database update drifts them silently by an hour.

Two things are already right. The `tzdb` feature bundles the IANA database inside the binary through jiff (`tzdb-bundle-always`, no `tz-system`), so no system zone file is ever read and two machines on one crate version resolve alike. And offset resolution from the calendar's own `VTIMEZONE` needs no database at all (tz: offset resolution from the calendar's own rules).

What is missing, borrowed from an external engine that stamps every materialised occurrence with the database version it was expanded under:

- the version of the bundled database is not exposed, so a consumer cannot record it;
- a `TZID` the calendar does not define has no resolution: a consumer resolving a zoned time through the resource's own `VTIMEZONE`, else a time zone database by `TZID`, else as floating, is only served the first step here.

## What

Behind `tzdb`, and only there:

- `tzdb::version()`: the IANA release the bundled database holds (`2026c` today), so a consumer stamps what it expanded with it and re-expands when it moves.
- `tzdb::zone(tzid, through)`: an `IcalTz` built from the database for an IANA name, usable wherever an `IcalTz` from a `VTIMEZONE` is (`IcalRecurSet::of_component_in`, `expand_in_zone`, the instant walk of occurrences-as-instants).
- `tzdb::zones(components, through)`: the zones a calendar's components resolve through, in the order "the calendar's own `VTIMEZONE`, else the database, else floating", in one call. The calendar's own definition always wins: the database is a fallback for what the sender omitted, never an override.
- Beside it, in the core, `IcalTz::of_components`: the calendar's own zones alone, the same list with no database.
- Determinism stated: one crate version and one lock file, one event, one window give the same instants on every machine.

## Decisions

- **The version** is read from `jiff-tzdb`'s `VERSION`, the crate jiff's `tzdb-bundle-always` bundles, taken as a direct optional dependency of the `tzdb` feature at the same `0.1` requirement, so Cargo unifies the two into one crate and the name read is the one of the data resolved. jiff itself exposes no version. `version()` answers `Option<&'static str>`, `None` only for a bundle generated without one, which `jiff-tzdb` allows.
- **The fallback zone is an `IcalTz`**, not a resolver of its own, so every place that takes a zone takes it unchanged. It holds every transition the database states up to the end of a given year (`through`), each as a dated onset (an `RDATE`, not a rule), grouped into one observance per pair of offsets. That is exact over the whole history up to the bound, unlike `tzdb::vtimezone`, which describes one era by a yearly rule for a document to carry. Past the bound the last offset holds, so a caller passes the last year it reads in, plus one.
- **Windows zone names** are out of scope: `Pacific Standard Time` maps onto an IANA name through CLDR's `windowsZones.xml`, which jiff does not carry, and a time naming one reads floating as today.
- **Precedence by order**: the zone lists a set reads through already take the first zone of a name, so `zones` lists the calendar's own first and a database zone only for a name no `VTIMEZONE` defines.
