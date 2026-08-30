//! # RECUR value codec (RFC 5545 3.3.10)
//!
//! [`Codec`] for a recurrence rule. A RECUR value uses `;` to separate its
//! rule parts (`FREQ=DAILY;COUNT=10`), which the generic node reads as separate
//! components; the whole value is kept, separators and all, and written back
//! verbatim (unescaped, since RECUR is not TEXT) on encode.

use crate::{
    tree::{
        codec::{Codec, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::recur::IcalRecur,
};

impl<'v> Codec<'v> for IcalRecur<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalRecur(node.decode())
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        let mut node = IcalValueNode::parse(self.0.as_bytes()).into_static();
        node.escaper = escaper;
        node
    }
}
