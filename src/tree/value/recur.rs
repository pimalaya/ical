//! # RECUR value codec (RFC 5545 3.3.10)
//!
//! [`Codec`] for a recurrence rule. A RECUR value uses `;` to separate its
//! rule parts (`FREQ=DAILY;COUNT=10`), which the generic node reads as separate
//! components; they are rejoined with `;` on decode and written verbatim
//! (unescaped, since RECUR is not TEXT) on encode.

use crate::{
    tree::{
        codec::{Codec, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::recur::IcalRecur,
};

use alloc::{borrow::Cow, string::String};

impl<'v> Codec<'v> for IcalRecur<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        let mut joined = String::new();

        for i in 0..node.component_count() {
            if i > 0 {
                joined.push(';');
            }
            joined.push_str(&node.decode_joined_at(i));
        }

        IcalRecur(Cow::Owned(joined))
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        let mut node = IcalValueNode::parse(self.0.as_bytes()).into_static();
        node.escaper = escaper;
        node
    }
}
