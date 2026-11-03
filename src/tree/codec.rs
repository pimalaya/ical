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
    tree::{
        codec::{encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
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

    /// Encode by dispatching to the held value's own shape.
    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalValue<'_> {
    /// Write the held value's wire shape.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        match self {
            IcalValue::Binary(v) => v.emit(out),
            IcalValue::Boolean(v) => v.emit(out),
            IcalValue::CalAddress(v) => v.emit(out),
            IcalValue::Date(v) => v.emit(out),
            IcalValue::DateTime(v) => v.emit(out),
            IcalValue::DateTimeList(v) => v.emit(out),
            IcalValue::Duration(v) => v.emit(out),
            IcalValue::Float(v) => v.emit(out),
            IcalValue::Geo(v) => v.emit(out),
            IcalValue::Integer(v) => v.emit(out),
            IcalValue::Period(v) => v.emit(out),
            IcalValue::Recur(v) => v.emit(out),
            IcalValue::RequestStatus(v) => v.emit(out),
            IcalValue::Text(v) => v.emit(out),
            IcalValue::TextList(v) => v.emit(out),
            IcalValue::Time(v) => v.emit(out),
            IcalValue::Uri(v) => v.emit(out),
            IcalValue::UtcOffset(v) => v.emit(out),
            IcalValue::Unknown(v) => v.emit(out),
        }
    }
}
