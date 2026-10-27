---
cairn: log
change: tzdb-pinned-and-versioned
date: 2026-10-27
---

# The bundled time zone database names its version and resolves what a calendar omits

pimdir's search index materialises calendar occurrences as instants (SEARCH §7), resolving a zoned time through the resource's own `VTIMEZONE`, else a time zone database by `TZID`, else as floating, and re-expanding when the database changes release. The crate served the first step only, and named no release.

- `tzdb::version() -> Option<&'static str>` reads `jiff_tzdb::VERSION` (`2026c` with jiff-tzdb 0.1.8). jiff exposes no version itself, so `jiff-tzdb` is a direct optional dependency of `tzdb` at jiff's own `0.1` requirement, which Cargo unifies into the one crate jiff bundles: the lock file holds a single `jiff-tzdb`.
- `tzdb::zone(tzid, through) -> Option<IcalTz>` builds a zone from every transition the database states up to the end of year `through`, each a dated onset of the observance between its two offsets, so it is exact over the whole history (New York's 2007 rule change included) and fits wherever a `VTIMEZONE`-read zone does. `tzdb::vtimezone` keeps describing one era by a yearly rule, which is what a document should carry; this one is for resolving.
- `tzdb::zones(components, through) -> Vec<IcalTz>` lists the calendar's own zones, then a database zone for each `TZID` the components name and leave undefined. Every reader takes the first zone of a name, so the calendar's own definition wins.
- `IcalTz::of_components` is the core's half: the calendar's own zones, no database.

Windows zone names (`Pacific Standard Time`) stay out of scope: their IANA mapping is CLDR's, which jiff does not carry, and such a time reads floating.

The bare core stays dependency-free: `cargo tree --no-default-features -e normal` lists the crate alone.

Capability moved: **tz** (the bundled database names its version; a zone the calendar omits resolves from the database).
