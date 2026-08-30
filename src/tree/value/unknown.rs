//! # Unknown value codec
//!
//! [`Codec`] for a value the model does not decode: its raw components are kept
//! (unescaped on read, re-escaped on write) so anything round-trips.

use crate::{
    tree::{
        codec::{Codec, encode::encode_component, mode::Escaper},
        value::node::IcalValueNode,
    },
    value::IcalUnknownValue,
};

impl<'v> Codec<'v> for IcalUnknownValue<'v> {
    fn decode(node: &'v IcalValueNode<'_>) -> Self {
        IcalUnknownValue {
            components: (0..node.component_count())
                .map(|i| node.decode_component_list(i))
                .collect(),
        }
    }

    fn encode(&self, escaper: Escaper) -> IcalValueNode<'static> {
        IcalValueNode::from_components(
            self.components
                .iter()
                .map(|component| encode_component(component, escaper))
                .collect(),
            escaper,
        )
    }
}
