# 📅 ical-rs [![Documentation](https://img.shields.io/docsrs/ical-rs?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/ical-rs/latest/ical) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

[iCalendar](https://www.rfc-editor.org/rfc/rfc5545) Rust library

`ical-rs` reads, edits and writes calendar objects in both iCalendar flavours: vCalendar 1.0 (versit) and iCalendar 2.0 ([RFC 5545](https://www.rfc-editor.org/rfc/rfc5545), extended by [7986](https://www.rfc-editor.org/rfc/rfc7986), [7529](https://www.rfc-editor.org/rfc/rfc7529), [9073](https://www.rfc-editor.org/rfc/rfc9073) and [9074](https://www.rfc-editor.org/rfc/rfc9074)). It treats them interchangeably, so you never pick a dialect up front, and it preserves the nested structure of a calendar (events, alarms, time zones) as it is.

Its defining trait is faithful editing: change one field of a parsed calendar and everything you did not touch, down to the exact bytes and the line endings, comes back unchanged. It is forgiving on the way in, accepting any real-world calendar including its odd or vendor-specific parts, and strict on the way out when you ask for it, building and checking a calendar against the standard so you know it conforms before you send it.

## Table of contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Benchmarks](#benchmarks)
- [License](#license)
- [AI disclosure](#ai-disclosure)
- [Contributing](CONTRIBUTING.md)
- [Social](#social)
- [Sponsoring](#sponsoring)

## Features

- **Every version, one library**: read, edit and write vCalendar 1.0 and iCalendar 2.0 without choosing a dialect; the version travels with the calendar.
- **Nested structure, preserved**: a calendar is a tree of components (events, to-dos, journals, free/busy, time zones, alarms); the whole tree is parsed, walked and round-tripped.
- **Faithful editing**: change a field and every untouched part of the calendar is preserved exactly, so a round-trip never rewrites what you did not mean to change. A value written in a foreign character set survives unaltered.
- **Forgiving on input**: any real calendar is accepted, including components, properties, parameters and value types the library has never heard of, so nothing is silently dropped.
- **Strict on output when you want it**: build a calendar guided by the standard and check it for conformance, or take the escape hatch and assemble one by hand with no checks.
- **Small and portable**: runs in constrained environments with no operating system, and stays lean, pulling in nothing beyond what the optional decoders you enable need.
- **Optional decoders**: opt in to decode encoded text, inline binary data such as attachments, and text in foreign character sets.

## Installation

```toml
[dependencies]
ical-rs = "0.0.1"
```

## Usage

The snippets below are condensed; full runnable versions live in [`examples/`](examples), each launchable with `cargo run --example <name>`.

### Parse a calendar, edit a field, and write it back

Only the field you touch changes; every other byte of the original calendar, including the line endings and the parameters you did not edit, round-trips exactly.

```rust
use ical::tree::cst::IcalCst;
use ical::tree::prop::summary::SUMMARY;

let mut cal = IcalCst::parse(
    "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\nBEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\nSUMMARY:Lunch\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
).unwrap();

cal.component_mut::<ical::tree::component::vevent::VEVENT>()
    .unwrap()
    .prop_mut::<SUMMARY>()
    .unwrap()
    .set_text("Dinner");
```

### Build a calendar, checked against the standard

Each property is checked as it is built, and the finished calendar is validated as a whole before it is written out; a calendar that does not conform gives you the list of problems instead. See [`examples/strict_builder.rs`](examples/strict_builder.rs).

### Build a calendar by hand, unchecked

The escape hatch: place whatever properties and components you like and write them out directly, with no validation. Correctness is your responsibility. See [`examples/raw_builder.rs`](examples/raw_builder.rs).

Beyond parsing and building, the library projects a calendar onto a decoded model and back, and, behind opt-in features, decodes encoded text, inline binary data and foreign character sets.

## Benchmarks

Single-object [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse`. The suite measures this crate's own stages over a realistic `VCALENDAR` with a nested `VEVENT` and `VALARM`: parsing to the byte-faithful CST, decoding onto the model, encoding back to bytes, and a full round-trip.

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## AI disclosure

This project is developed with AI assistance. This section documents how, so users and downstream packagers can make informed decisions.

- **Tools**: Claude Code (Anthropic), Opus 4.8, invoked locally with a persistent project-scoped memory and a small set of repo-specific rules.
- **Used for**: Refactors, mechanical multi-file edits, boilerplate (feature gates, error enums, derive macros, trait impls), test scaffolding, doc polish, exploratory design conversations.
- **Not used for**: Engineering, critical code, git manipulation (commit, merge, rebase…), real-world tests.
- **Verification**: Every AI-assisted change is read, compiled, tested, and formatted before commit (`nix develop --command cargo check / cargo test / cargo fmt`). Behavioural correctness is verified against the relevant RFC or upstream spec, not assumed from the model output. Tests are never adjusted to fit AI-generated code; the code is adjusted to fit correct behaviour.
- **Limitations**: AI models occasionally produce code that compiles and passes tests but is subtly wrong: off-by-one errors, missed edge cases, plausible but nonexistent APIs, stale RFC references. The verification workflow catches most of this; it does not catch all of it. Bug reports are welcome and taken seriously.
- **Last reviewed**: 02/07/2026

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
