//! # Codec
//!
//! The bytes-to-model bridge, in both directions and at both levels. The only
//! part of [`crate::tree`] that consults the calendar version.
//!
//! [`decode`] projects a raw syntax tree onto the decoded model and
//! [`encode`] projects it back; that is the structural level.
//!
//! Underneath, at the value-string level, [`escape`] and [`unescape`] apply
//! and resolve the RFC 5545 3.3.11 value escapes, keyed by the [`mode`]
//! `Escaper`, and every value leaf the structural codecs touch runs through
//! them.
//!
//! A parameter value is a different alphabet, with no backslash escapes at
//! all (RFC 5545 3.2) and the RFC 6868 caret encoding instead, so it has its
//! own pair in the same two modules, keyed by the same `Escaper`.
//!
//! Content transfer encodings (`QUOTED-PRINTABLE`, `BASE64`) and `CHARSET`
//! are never resolved here: the core transforms no content, leaving that to
//! the opt-in feature helpers.
//!
//! The per-value-type projection is the [`Codec`] trait, implemented once per
//! value type under [`crate::tree::value`], mirroring the model's `value/`;
//! both the structural dispatch and the per-property lenses go through it.

pub mod decode;
pub mod encode;
pub mod escape;
pub mod mode;
pub mod unescape;

use crate::{
    tree::{codec::mode::Escaper, value::node::IcalValueNode},
    value::{IcalUnknownValue, IcalValue},
};

/// How a decoded value type projects to and from a syntax node.
///
/// `decode` reads it from a node, whose [`escaper`](IcalValueNode::escaper)
/// carries the mode, and `encode` writes it back, escaping every leaf with the
/// target [`Escaper`] and stamping it on the node. The two stay symmetric, the
/// decoded value itself being escaper-agnostic clean text.
pub trait Codec<'v>: Sized {
    /// Decode the value from a syntax node.
    fn decode(node: &'v IcalValueNode<'_>) -> Self;

    /// Encode the value into a syntax node for the given escaping mode.
    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static>;
}

impl<'v> Codec<'v> for IcalValue<'v> {
    /// Decode liberally as raw [`Unknown`](IcalValue::Unknown).
    ///
    /// No value kind is known at this level, that being the spec's job, so the
    /// version-divergent lenses whose target is `IcalValue` override the lens
    /// `decode` to resolve the real kind; this fallback is what others inherit.
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalValue::Unknown(IcalUnknownValue::decode(node))
    }

    /// Encode by dispatching to the held value's own codec.
    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        match self {
            IcalValue::Binary(v) => v.encode(escaper),
            IcalValue::Boolean(v) => v.encode(escaper),
            IcalValue::CalAddress(v) => v.encode(escaper),
            IcalValue::Date(v) => v.encode(escaper),
            IcalValue::DateTime(v) => v.encode(escaper),
            IcalValue::DateTimeList(v) => v.encode(escaper),
            IcalValue::Duration(v) => v.encode(escaper),
            IcalValue::Float(v) => v.encode(escaper),
            IcalValue::Geo(v) => v.encode(escaper),
            IcalValue::Integer(v) => v.encode(escaper),
            IcalValue::Period(v) => v.encode(escaper),
            IcalValue::Recur(v) => v.encode(escaper),
            IcalValue::RequestStatus(v) => v.encode(escaper),
            IcalValue::Text(v) => v.encode(escaper),
            IcalValue::TextList(v) => v.encode(escaper),
            IcalValue::Time(v) => v.encode(escaper),
            IcalValue::Uri(v) => v.encode(escaper),
            IcalValue::UtcOffset(v) => v.encode(escaper),
            IcalValue::Unknown(v) => v.encode(escaper),
        }
    }
}
