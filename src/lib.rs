#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # ical-rs
//!
//! One version-agnostic iCalendar library: a decoded model and a
//! byte-faithful syntax tree that read and write vCalendar 1.0 (versit) and
//! iCalendar 2.0 (RFC 5545, extended by 6638, 7529, 7953, 7986, 9073, 9074
//! and 9253) alike.
//!
//! The version is a decoded indicator, never a type parameter: the tree
//! ignores it, and only the codec and the per-property spec branch on it,
//! where escaping or a value's shape genuinely differ.
//!
//! Unlike a flat address book, a calendar is a tree of components (events,
//! to-dos, journals, free/busy, time zones, alarms), and the whole tree is
//! parsed, walked and round-tripped.
//!
//! The crate is `no_std` (with `alloc`), its core is dependency-free, and
//! every dependency sits behind a [cargo feature](#cargo-features).
//!
//! This header is the architecture; the behaviour behind it is specified
//! capability by capability in the repository's cairn/spec folder.
//!
//! ## Postel's law
//!
//! Parsing is maximally liberal: any real calendar round-trips byte for byte,
//! components, properties, parameters and value types no version defines
//! included, and an `Unknown` arm on every open vocabulary carries that
//! openness into the model.
//!
//! Strictness lives on the way out: the [`builder`] refuses to construct a
//! property the spec forbids, and the [`validator`] checks a decoded calendar
//! against its version's RFC contract. Neither needs the parser.
//!
//! ## The two layers
//!
//! The decoded model ([`ical`], [`component`], [`version`], [`prop`],
//! [`param`], [`value`]) is pure data with no dependency on the syntax side,
//! so it can be depended on alone, and so can everything reading it: the
//! [`builder`], the [`validator`], [`recur`], [`tz`] and both JSON
//! representations.
//!
//! Component, property and parameter names and value types are closed
//! identity enums ([`IcalComponentKind`], [`IcalPropKind`],
//! [`IcalParamKind`], [`IcalValueKind`]) whose wire spelling is reached
//! through `FromStr` and `Deref`.
//!
//! A calendar is an [`Ical`]: a version, the calendar-level properties, and a
//! list of nested [`IcalComponent`]s, themselves recursive.
//!
//! A property is an [`IcalProp`] of a name, parameters and one value, the
//! last two open payload enums ([`IcalParam`], [`IcalValue`]) with an
//! `Unknown` arm, so anything outside the model survives.
//!
//! The syntax tree ([`tree`], behind the default `parser` feature) is
//! everything byte-faithful. Its hub is [`IcalCst`], a recursive tree of
//! generic nodes reproducing the wire bytes exactly.
//!
//! Exactly means exactly: the tokeniser resolves a line's wire layout (its
//! RFC 5545 3.1 folds, the blank lines before it, its `QUOTED-PRINTABLE` soft
//! breaks) so every layer above sees one logical line, and records it on
//! [`IcalWire`] so serialization lays it back out.
//!
//! Only an edit that changes a line's length drops that layout, since the
//! recorded fold points no longer index the bytes they were taken against.
//!
//! [`parse`] reads one calendar and [`parse_many`] iterates a multi-calendar
//! file, both strict, refusing a calendar they cannot structure.
//!
//! [`parse_recovering`] keeps what it cannot structure as opaque bytes,
//! carries on, and reports what it worked around, for the calendars in the
//! wild that a strict reading throws away whole.
//!
//! [`decode`] projects a CST onto the decoded [`Ical`], and `encode` (with
//! `From<Ical>`) projects the model back to a canonical CST.
//!
//! A per-property lens ([`IcalPropLens`], implemented on the marker the
//! property defines in [`prop`]) reads or edits one line through the
//! byte-preserving [`cursor`]s, so editing one property leaves every other
//! byte intact. A component marker keys the same access over a whole subtree.
//!
//! ## The spec layer
//!
//! Each property carries an [`IcalPropSpec`] on its marker (the versions it
//! lives in, its cardinality, the value types and parameters it may take per
//! version), and each component an [`IcalComponentSpec`] (the children it may
//! nest and the properties it requires).
//!
//! A contract is what the RFC allows, so it is model rather than syntax: the
//! markers live in [`prop`] and [`component`], and only their read-and-edit
//! lens sits under [`tree`].
//!
//! One vtable dispatch bridges the open kinds back to those static specs, so
//! the decoder, the [`validator`] and the [`builder`] all consult one source
//! of truth.
//!
//! A calendar that passes earns an [`IcalValid`](validator::IcalValid) proof,
//! and both `Ical` and `IcalValid<Ical>` convert back into an [`IcalCst`].
//!
//! ## Recurrence and time zones
//!
//! [`recur`] answers what a rule denotes, and what a whole component denotes:
//! [`IcalRecurExpand`] walks one `RRULE`, and [`IcalRecurSet`] walks the set
//! an event actually happens on, `RDATE`s, `EXDATE`s, `EXRULE`s and
//! `RECURRENCE-ID` overrides included.
//!
//! Both are lazy, and both are civil: RFC 5545 expands on the local
//! wall-clock time of `DTSTART`, so no offset is ever needed and none is ever
//! resolved.
//!
//! [`tz`] is the step after, turning a civil occurrence into a UTC
//! offset from the `VTIMEZONE` the calendar carries, and reporting the
//! spring-forward gap and the fall-back fold rather than guessing.
//!
//! ## Reconciling two replicas
//!
//! [`merge`](tree::merge) is the syntax layer's answer to two divergent edits
//! of one calendar: [`IcalMerge`] diffs each against their common base,
//! reports what each did and where they collided, and builds the merged
//! calendar out of the left side's own bytes.
//!
//! It lives under [`tree`] rather than over the model because keeping the
//! bytes of every line neither side touched is the point.
//!
//! ## The JSON representations
//!
//! [`jcal`] is the RFC 7265 spelling of this model in JSON, member for
//! member.
//!
//! [`jscalendar`] is the RFC 8984 data model, which is a different model: a
//! `VCALENDAR` is a Group of Events and Tasks, a `DTEND` is a duration, an
//! `ATTENDEE` line is a Participant object, and an overriding `VEVENT` is a
//! patch inside the series it overrides.
//!
//! Both are lossless, each through an escape hatch of its own, and both take
//! a raw [`serde_json::Value`] at the boundary rather than a serde
//! implementation, since one model with two JSON spellings is exactly what
//! serde cannot key.
//!
//! ## Cargo features
//!
//! `parser` (default) brings the byte-faithful [`tree`] and its codec, via
//! the `memchr` crate. Everything under [`tree`] is gated on it; the decoded
//! model, the builder, the validator, the recurrence layer, the time zones
//! and both JSON representations are always available.
//!
//! Three content decoders are default too, one small crate each:
//! `quoted-printable` decodes `QUOTED-PRINTABLE` value octets, `base64`
//! decodes inline `BASE64` binary values, and `encoding` transcodes a foreign
//! `CHARSET` to text through `encoding_rs` (the WHATWG Encoding Standard).
//!
//! `jcal` adds the RFC 7265 JSON representation, via the `serde_json` crate.
//! `jscalendar` adds the RFC 8984 JSON data model, implies `jcal`, whose
//! syntax carries the escape hatch, and pulls no crate of its own.
//!
//! [`IcalComponentKind`]: component::IcalComponentKind
//! [`IcalPropKind`]: prop::IcalPropKind
//! [`IcalParamKind`]: param::IcalParamKind
//! [`IcalValueKind`]: value::IcalValueKind
//! [`Ical`]: ical::Ical
//! [`IcalComponent`]: component::IcalComponent
//! [`IcalProp`]: prop::IcalProp
//! [`IcalParam`]: param::IcalParam
//! [`IcalValue`]: value::IcalValue
//! [`IcalCst`]: tree::cst::IcalCst
//! [`IcalWire`]: tree::wire::IcalWire
//! [`parse`]: tree::cst::IcalCst::parse
//! [`parse_many`]: tree::cst::IcalCst::parse_many
//! [`parse_recovering`]: tree::cst::IcalCst::parse_recovering
//! [`decode`]: tree::cst::IcalCst::decode
//! [`IcalPropLens`]: tree::prop::lens::IcalPropLens
//! [`cursor`]: tree::value::cursor::IcalValueCursor
//! [`IcalPropSpec`]: prop::spec::IcalPropSpec
//! [`IcalComponentSpec`]: component::spec::IcalComponentSpec
//! [`IcalRecurExpand`]: recur::expand::IcalRecurExpand
//! [`IcalRecurSet`]: recur::set::IcalRecurSet
//! [`IcalMerge`]: tree::merge::IcalMerge

extern crate alloc;

pub mod builder;
pub mod component;
pub mod ical;
pub mod param;
pub mod prop;
pub mod recur;
pub mod tz;
pub mod validator;
pub mod value;
pub mod version;

#[cfg(feature = "jcal")]
#[cfg_attr(docsrs, doc(cfg(feature = "jcal")))]
pub mod jcal;

#[cfg(feature = "jscalendar")]
#[cfg_attr(docsrs, doc(cfg(feature = "jscalendar")))]
pub mod jscalendar;

#[cfg(feature = "parser")]
pub mod tree;
