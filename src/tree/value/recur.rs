//! # RECUR value codec (RFC 5545 3.3.10)
//!
//! [`Codec`] for a recurrence rule. A RECUR value uses `;` to separate its
//! rule parts (`FREQ=DAILY;COUNT=10`), which the generic node reads as separate
//! components; the whole value is kept, separators and all, and written back
//! verbatim (unescaped, since RECUR is not TEXT) on encode.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::recur::IcalRecur,
};

impl<'v> Codec<'v> for IcalRecur<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalRecur(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalRecur<'_> {
    /// Write the rule byte for byte, its separators in place.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        out.raw(&self.0);
    }
}
