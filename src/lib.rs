#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # ical-rs
//!
//! A single, version-agnostic iCalendar library: one decoded model and one
//! byte-faithful syntax tree that read and write vCalendar 1.0 (versit) and
//! iCalendar 2.0 (RFC 5545, extended by 7986, 7529, 9073 and 9074) alike. The
//! calendar version is a decoded indicator, never a type parameter or a
//! separate dialect: the syntax tree ignores it, and only the codec and the
//! per-property spec branch on it where escaping or a value's shape genuinely
//! differ. Unlike a flat address book, a calendar is a tree of components
//! (events, to-dos, journals, free/busy, time zones, alarms), and the whole
//! tree is parsed, walked and round-tripped. The crate is `no_std` (with
//! `alloc`); its core is dependency-free, and the only dependencies are the
//! small `no_std` crates behind the opt-in content-decoding features
//! ([Cargo features](#cargo-features)).
//!
//! ## Postel's law
//!
//! The library is liberal in what it accepts and strict in what it sends.
//! Parsing is maximally liberal: any real calendar, including components,
//! properties, parameters and value types that no version officially defines,
//! is accepted and round-trips byte for byte. The decoded model keeps that
//! openness, with an `Unknown` arm on every open vocabulary. Strictness lives
//! only on the way out, as two runtime steps: the builder, which refuses to
//! construct a property the spec forbids, and
//! [`validate`](tree::ical::validate), which checks a decoded calendar against
//! its version's RFC contract.
//!
//! ## The two layers
//!
//! The decoded model ([`ical`], [`component`], [`version`], [`prop`], [`param`],
//! [`value`]) is pure data with no dependency on the syntax side, so it can be
//! depended on alone. Component names, property names, parameter names and value
//! types are closed identity enums
//! ([`IcalComponentKind`](component::IcalComponentKind),
//! [`IcalPropKind`](prop::IcalPropKind),
//! [`IcalParamKind`](param::IcalParamKind),
//! [`IcalValueKind`](value::IcalValueKind)) whose wire spelling is reached
//! through `FromStr` and `Deref`. A calendar is an [`Ical`](ical::Ical): a
//! version, the calendar-level properties, and a list of nested
//! [`IcalComponent`](component::IcalComponent)s (themselves recursive). A
//! property is a [`IcalProp`](prop::IcalProp) of a name, parameters and one
//! value; its parameters and value are open payload enums
//! ([`IcalParam`](param::IcalParam), [`IcalValue`](value::IcalValue)) with an
//! `Unknown` variant, so anything outside the model survives.
//!
//! The syntax tree ([`tree`], gated behind the `parser` feature, on by default)
//! is everything byte-faithful. Its hub is [`IcalCst`](tree::cst::IcalCst), a
//! recursive tree of generic nodes that reproduces the wire bytes exactly.
//! [`parse`](tree::cst::IcalCst::parse) accepts bytes or a string and reads one
//! calendar; [`parse_many`](tree::cst::IcalCst::parse_many) iterates a
//! multi-calendar file. [`decode`](tree::cst::IcalCst::decode) projects a CST
//! onto the decoded [`Ical`](ical::Ical); `encode` (and `From<Ical>`) projects
//! the model back to a canonical CST. Per-property lens markers
//! ([`IcalPropLens`](tree::prop::IcalPropLens)) and per-component lens markers
//! ([`IcalComponentLens`](tree::component::IcalComponentLens)) read or edit a
//! single line or subtree through the byte-preserving
//! [`cursor`](tree::value::IcalValueCursor)s, so editing one property leaves
//! every other byte intact.
//!
//! ## The spec layer
//!
//! Each property carries a [`IcalPropSpec`](tree::prop::IcalPropSpec) on its
//! lens marker (the versions it lives in, its cardinality, the value types and
//! parameters it may take per version), and each component carries a
//! [`IcalComponentSpec`](tree::component::IcalComponentSpec) (its allowed parent
//! and child components, and its required and optional properties). A single
//! vtable dispatch bridges the open kinds back to those static specs, so the
//! decoder, [`validate`](tree::ical::validate) and the builder all consult one
//! source of truth. A calendar that passes earns a
//! [`Valid`](tree::ical::validate::Valid) proof, and both `Ical` and
//! `Valid<Ical>` convert back into a [`IcalCst`](tree::cst::IcalCst).
//!
//! ## Cargo features
//!
//! - `parser` (default): the byte-faithful [`tree`] and its codec. Everything
//!   under [`tree`] is gated on it; the decoded model is always available.
//! - `quoted-printable` (default): decode `QUOTED-PRINTABLE` value octets, via
//!   the `quoted_printable` crate.
//! - `base64` (default): decode inline `BASE64` binary values, via the `base64`
//!   crate.
//! - `encoding` (default): transcode a foreign `CHARSET` value to text, via the
//!   `encoding_rs` crate (the WHATWG Encoding Standard).

extern crate alloc;

pub mod component;
pub mod ical;
pub mod param;
pub mod prop;
pub mod value;
pub mod version;

#[cfg(feature = "parser")]
pub mod tree;
