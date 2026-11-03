//! # Unknown value codec
//!
//! [`Codec`] for a value the model does not decode: its raw components are kept
//! (unescaped on read, re-escaped on write) so anything round-trips.

use crate::{
    tree::{
        codec::{Codec, encode::IcalValueOut, mode::Escaper},
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
        IcalValueOut::node(escaper, |out| self.emit(out))
    }
}

impl IcalUnknownValue<'_> {
    /// Write every component, each a list of its values.
    pub(crate) fn emit(&self, out: &mut IcalValueOut<'_>) {
        for component in &self.components {
            out.component(component);
        }
    }
}
